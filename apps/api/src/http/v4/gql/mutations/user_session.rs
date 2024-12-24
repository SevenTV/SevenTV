use std::sync::Arc;

use async_graphql::Context;
use shared::database::role::permissions::{PermissionsExt, UserPermission};
use shared::database::stored_event::StoredEventUserSessionData;
use shared::database::user::session::UserSession;
use shared::database::user::UserId;
use shared::event::{InternalEvent, InternalEventData};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::jwt::{AuthJwtPayload, JwtState};
use crate::transactions::{transaction, TransactionError};

#[derive(Default)]
pub struct UserSessionMutation;

#[async_graphql::Object]
impl UserSessionMutation {
	#[tracing::instrument(skip_all, name = "UserSessionMutation::create")]
	async fn create(
		&self,
		ctx: &Context<'_>,
		user_id: UserId,
		expires_at: chrono::DateTime<chrono::Utc>,
	) -> Result<String, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		if !session.has(UserPermission::ManageSessions) {
			return Err(ApiError::forbidden(ApiErrorCode::LackingPrivileges, "lacking privileges"));
		}

		global
			.user_by_id_loader
			.load(user_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

		let res = transaction(global, |mut tx| async move {
			let user_session = UserSession {
				id: Default::default(),
				user_id,
				expires_at,
				last_used_at: chrono::Utc::now(),
				extensions: Default::default(),
			};

			tx.insert_one::<UserSession>(&user_session, None).await?;

			tx.register_event(InternalEvent {
				actor: Some(authed_user.clone()),
				session_id: session.user_session_id(),
				data: InternalEventData::UserSession {
					after: user_session.clone(),
					data: StoredEventUserSessionData::Create { platform: None },
				},
				timestamp: chrono::Utc::now(),
			})?;

			// create jwt access token
			let jwt = AuthJwtPayload::from(user_session);
			let token = jwt
				.serialize(global)
				.ok_or_else(|| {
					tracing::error!("failed to serialize jwt");
					ApiError::internal_server_error(ApiErrorCode::Unknown, "failed to serialize jwt")
				})
				.map_err(TransactionError::Custom)?;

			Ok(token)
		})
		.await;

		match res {
			Ok(token) => Ok(token),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}
}
