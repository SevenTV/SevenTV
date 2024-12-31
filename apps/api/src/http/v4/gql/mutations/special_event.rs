use std::sync::Arc;

use async_graphql::Context;
use shared::database::product::special_event::SpecialEventId;
use shared::database::role::permissions::UserPermission;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::PermissionGuard;
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::SpecialEvent;

#[derive(Default)]
pub struct SpecialEventMutation;

#[derive(async_graphql::InputObject)]
struct CreateSpecialEventInput {
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
}

#[async_graphql::Object]
impl SpecialEventMutation {
	#[tracing::instrument(skip_all, name = "SpecialEventMutation::create")]
	#[graphql(guard = "PermissionGuard::one(UserPermission::ManageBilling)")]
	async fn create(&self, ctx: &Context<'_>, data: CreateSpecialEventInput) -> Result<SpecialEvent, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;
		let created_by = session.user()?;

		let special_event = shared::database::product::special_event::SpecialEvent {
			id: SpecialEventId::default(),
			name: data.name,
			description: data.description,
			tags: data.tags,
			created_by: created_by.id,
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		};

		shared::database::product::special_event::SpecialEvent::collection(&global.db)
			.insert_one(&special_event)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to create redeem code");
				ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to create redeem code")
			})?;

		Ok(special_event.into())
	}
}
