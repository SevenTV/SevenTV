use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use hyper::{header, StatusCode};
use mongodb::bson::doc;
use shared::database::{Collection, Permissions, User, UserId, UserSession};
use tokio::sync::OnceCell;

use super::cookies::Cookies;
use crate::dataloader::user_loader::load_user_and_permissions;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::jwt::{AuthJwtPayload, JwtState};

pub const AUTH_COOKIE: &str = "seventv-auth";

#[derive(Debug, Clone)]
pub struct AuthSession {
	pub kind: AuthSessionKind,
	/// lazy user data
	cached_data: Arc<OnceCell<(User, Permissions)>>,
}

#[derive(Debug, Clone)]
pub enum AuthSessionKind {
	/// The user session
	Session(UserSession),
	/// Old user sessions, only user id available
	Old(UserId),
}

impl AuthSession {
	pub fn user_id(&self) -> UserId {
		match &self.kind {
			AuthSessionKind::Session(session) => session.user_id,
			AuthSessionKind::Old(user_id) => *user_id,
		}
	}

	/// Lazy load user data
	pub async fn user(&self, global: &Arc<Global>) -> Result<&(User, Permissions), ApiError> {
		self.cached_data
			.get_or_try_init(|| async {
				Ok(load_user_and_permissions(global, self.user_id())
					.await?
					.ok_or(ApiError::UNAUTHORIZED)?)
			})
			.await
	}
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AuthSession
where
	Arc<Global>: FromRef<S>,
	S: Send + Sync,
{
	type Rejection = ApiError;

	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		let global = Arc::<Global>::from_ref(state);

		let cookies = parts.extensions.get::<Cookies>().expect("cookies not found");
		let auth_cookie = cookies.get(AUTH_COOKIE);

		let Some(token) = auth_cookie.as_ref().map(|c| c.value()).or_else(|| {
			parts
				.headers
				.get(header::AUTHORIZATION)
				.and_then(|v| v.to_str().ok())
				.map(|s| s.trim_start_matches("Bearer "))
		}) else {
			return Err(ApiError::new(StatusCode::UNAUTHORIZED, "missing token"));
		};

		let jwt = AuthJwtPayload::verify(&global, token).ok_or_else(|| {
			cookies.remove(AUTH_COOKIE);
			ApiError::new(StatusCode::UNAUTHORIZED, "invalid token")
		})?;

		match jwt.session_id {
			Some(session_id) => {
				let session = UserSession::collection(global.db())
					.find_one_and_update(
						doc! {
							"_id": session_id,
							"expires_at": { "$gt": chrono::Utc::now() },
						},
						doc! {
							"$set": {
								"last_used_at": chrono::Utc::now(),
							},
						},
						Some(
							mongodb::options::FindOneAndUpdateOptions::builder()
								.return_document(mongodb::options::ReturnDocument::After)
								.upsert(false)
								.build(),
						),
					)
					.await
					.map_err(|err| {
						tracing::error!(error = %err, "failed to find user session");
						ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "failed to find user session")
					})?
					.ok_or_else(|| {
						cookies.remove(AUTH_COOKIE);
						ApiError::new(StatusCode::UNAUTHORIZED, "session not found")
					})?;

				Ok(AuthSession {
					kind: AuthSessionKind::Session(session),
					cached_data: Arc::new(OnceCell::new()),
				})
			}
			// old session
			None => Ok(AuthSession {
				kind: AuthSessionKind::Old(jwt.user_id),
				cached_data: Arc::new(OnceCell::new()),
			}),
		}
	}
}
