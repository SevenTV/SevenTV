use std::sync::Arc;

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use futures::future::BoxFuture;
use hyper::{header, StatusCode};
use shared::database::role::permissions::{
	FlagPermission, Permission, Permissions, PermissionsExt, RateLimitResource, UserPermission,
};
use shared::database::user::session::{UserSession, UserSessionId};
use shared::database::user::{FullUser, UserComputed, UserId};

use super::cookies::Cookies;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::jwt::{AuthJwtPayload, JwtState};
use crate::ratelimit::{RateLimitRequest, RateLimitResponse};

pub const AUTH_COOKIE: &str = "seventv-auth";

#[derive(Debug, Clone)]
pub struct Session(Arc<AuthState>, std::net::IpAddr);

#[derive(Debug, Clone)]
enum AuthState {
	Authenticated { kind: AuthSessionKind, user: Box<FullUser> },
	Unauthenticated { default: Box<UserComputed> },
}

#[derive(Debug, Clone)]
pub enum AuthSessionKind {
	/// The user session
	Session(UserSession),
	/// Old user sessions, only user id available
	Old(UserId),
}

impl AuthSessionKind {
	pub fn user_id(&self) -> UserId {
		match self {
			AuthSessionKind::Session(session) => session.user_id,
			AuthSessionKind::Old(user_id) => *user_id,
		}
	}
}

impl Session {
	pub fn user_id(&self) -> Option<UserId> {
		match &*self.0 {
			AuthState::Authenticated { user, .. } => Some(user.id),
			AuthState::Unauthenticated { .. } => None,
		}
	}

	pub fn can_view(&self, user: &FullUser) -> bool {
		!user.has(FlagPermission::Hidden) || self.has(UserPermission::ViewHidden) || Some(user.id) == self.user_id()
	}

	pub fn user_session(&self) -> Option<&UserSession> {
		match &*self.0 {
			AuthState::Authenticated {
				kind: AuthSessionKind::Session(session),
				..
			} => Some(session),
			_ => None,
		}
	}

	pub fn user_session_id(&self) -> Option<UserSessionId> {
		self.user_session().map(|s| s.id)
	}

	pub const fn ip(&self) -> std::net::IpAddr {
		self.1
	}

	pub fn user(&self) -> Option<&FullUser> {
		match &*self.0 {
			AuthState::Authenticated { user, .. } => Some(user),
			AuthState::Unauthenticated { .. } => None,
		}
	}

	pub fn permissions(&self) -> &Permissions {
		match &*self.0 {
			AuthState::Authenticated { user, .. } => &user.computed.permissions,
			AuthState::Unauthenticated { default } => &default.permissions,
		}
	}
}

impl PermissionsExt for Session {
	fn has(&self, permission: impl Into<Permission>) -> bool {
		match &*self.0 {
			AuthState::Authenticated { user, .. } => user.has(permission),
			AuthState::Unauthenticated { default } => default.has(permission),
		}
	}

	fn denied(&self, permission: impl Into<Permission>) -> bool {
		match &*self.0 {
			AuthState::Authenticated { user, .. } => user.denied(permission),
			AuthState::Unauthenticated { default } => default.denied(permission),
		}
	}
}

#[derive(Clone)]
pub struct SessionMiddleware(Arc<Global>);

impl SessionMiddleware {
	pub fn new(global: Arc<Global>) -> Self {
		Self(global)
	}
}

impl<S> tower::Layer<S> for SessionMiddleware {
	type Service = SessionMiddlewareService<S>;

	fn layer(&self, inner: S) -> Self::Service {
		SessionMiddlewareService {
			inner,
			global: self.0.clone(),
		}
	}
}

#[derive(Clone)]
pub struct SessionMiddlewareService<S> {
	inner: S,
	global: Arc<Global>,
}

impl<S> SessionMiddlewareService<S> {
	async fn modify_request<B>(&self, req: &mut Request<B>) -> Result<Option<RateLimitResponse>, ApiError> {
		let cookies = req.extensions().get::<Cookies>().expect("cookies not found");
		let ip = *req.extensions().get::<std::net::IpAddr>().expect("ip not found");
		let cookie = cookies.get(AUTH_COOKIE);

		let session = if let Some(token) = cookie.as_ref().map(|c| c.value()).or_else(|| {
			req.headers()
				.get(header::AUTHORIZATION)
				.and_then(|v| v.to_str().ok())
				.map(|s| s.trim_start_matches("Bearer "))
		}) {
			let token = token.trim_matches('\"');

			let jwt = AuthJwtPayload::verify(&self.global, token).ok_or_else(|| {
				cookies.remove(&self.global, AUTH_COOKIE);
				ApiError::new(StatusCode::UNAUTHORIZED, "invalid token")
			})?;

			let kind = match jwt.session_id {
				Some(session_id) => {
					let session = self
						.global
						.user_session_by_id_loader
						.load(session_id)
						.await
						.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
						.ok_or_else(|| {
							cookies.remove(&self.global, AUTH_COOKIE);
							ApiError::new_const(StatusCode::UNAUTHORIZED, "session not found")
						})?;

					if session.expires_at < chrono::Utc::now() {
						cookies.remove(&self.global, AUTH_COOKIE);
						return Err(ApiError::new_const(StatusCode::UNAUTHORIZED, "session expired"));
					}

					self.global.user_session_updater_batcher.load(session.id).await.ok();

					AuthSessionKind::Session(session)
				}
				// old session
				None => AuthSessionKind::Old(jwt.user_id),
			};

			let user = self
				.global
				.user_loader
				.load(&self.global, kind.user_id())
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or_else(|| {
					cookies.remove(&self.global, AUTH_COOKIE);
					ApiError::new_const(StatusCode::UNAUTHORIZED, "user not found")
				})?;

			Session(
				Arc::new(AuthState::Authenticated {
					kind,
					user: Box::new(user),
				}),
				ip,
			)
		} else {
			// Will load only the default permissions
			let default = self
				.global
				.user_loader
				.computed_loader
				.load(UserId::nil())
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or_else(|| {
					tracing::error!("failed to load default permissions");
					ApiError::INTERNAL_SERVER_ERROR
				})?;

			Session(
				Arc::new(AuthState::Unauthenticated {
					default: Box::new(default),
				}),
				ip,
			)
		};

		let ratelimit = self
			.global
			.rate_limiter
			.acquire(RateLimitRequest::new(RateLimitResource::Global, &session))
			.await?;

		req.extensions_mut().insert(session);

		Ok(ratelimit)
	}
}

impl<S, B, R> tower::Service<Request<B>> for SessionMiddlewareService<S>
where
	S: tower::Service<Request<B>, Response = R> + Clone + Send,
	S::Future: Send,
	S: Send + Sync + 'static,
	B: Send + 'static,
	R: IntoResponse,
{
	type Error = S::Error;
	type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
	type Response = Response;

	fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx).map_err(Into::into)
	}

	fn call(&mut self, mut req: Request<B>) -> Self::Future {
		let mut this = self.clone();

		Box::pin(async move {
			let rate_limit_resp = match this.modify_request(&mut req).await {
				Ok(rate_limit_resp) => rate_limit_resp,
				Err(err) => return Ok(err.into_response()),
			};

			let mut resp = this.inner.call(req).await?.into_response();

			if let Some(rate_limit_resp) = rate_limit_resp {
				resp.headers_mut().extend(rate_limit_resp.header_map());
			}

			Ok(resp)
		})
	}
}
