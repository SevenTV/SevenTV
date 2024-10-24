use std::sync::Arc;

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use futures::future::BoxFuture;
use hyper::header;
use shared::database::queries::filter;
use shared::database::role::permissions::{
	FlagPermission, Permission, Permissions, PermissionsExt, RateLimitResource, UserPermission,
};
use shared::database::stored_event::StoredEventUserSessionData;
use shared::database::user::session::{UserSession, UserSessionId};
use shared::database::user::{FullUser, UserComputed, UserId};
use shared::event::{InternalEvent, InternalEventData};

use super::cookies::Cookies;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::jwt::{AuthJwtPayload, JwtState};
use crate::ratelimit::{RateLimitRequest, RateLimitResponse};
use crate::transactions::{transaction, TransactionResult, TransactionSession};

pub const AUTH_COOKIE: &str = "seventv-auth";

#[derive(Debug, Clone)]
pub struct Session(Arc<AuthState>, std::net::IpAddr);

#[derive(Debug, Clone)]
enum AuthState {
	Authenticated { session: UserSession, user: Box<FullUser> },
	Unauthenticated { default: Box<UserComputed> },
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
			AuthState::Authenticated { session, .. } => Some(session),
			_ => None,
		}
	}

	pub fn user_session_id(&self) -> Option<UserSessionId> {
		self.user_session().map(|s| s.id)
	}

	pub const fn ip(&self) -> std::net::IpAddr {
		self.1
	}

	pub fn user(&self) -> Result<&FullUser, ApiError> {
		match &*self.0 {
			AuthState::Authenticated { user, .. } => Ok(user),
			AuthState::Unauthenticated { .. } => {
				Err(ApiError::unauthorized(ApiErrorCode::LoginRequired, "you are not logged in"))
			}
		}
	}

	pub fn permissions(&self) -> &Permissions {
		match &*self.0 {
			AuthState::Authenticated { user, .. } => &user.computed.permissions,
			AuthState::Unauthenticated { default } => &default.permissions,
		}
	}

	pub async fn logout(&self, global: &Arc<Global>) -> TransactionResult<(), ApiError> {
		transaction(global, |mut tx| async move { self.logout_with_tx(&mut tx).await }).await
	}

	pub async fn logout_with_tx(&self, tx: &mut TransactionSession<'_, ApiError>) -> TransactionResult<(), ApiError> {
		let Some(user_session) = self.user_session() else {
			return Ok(());
		};

		// is a new session
		let user_session = tx
			.find_one_and_delete(
				filter::filter! {
					UserSession {
						#[query(rename = "_id")]
						id: user_session.id,
					}
				},
				None,
			)
			.await?;

		if let Some(user_session) = user_session {
			tx.register_event(InternalEvent {
				actor: Some(self.user().cloned().unwrap_or_default()),
				session_id: Some(user_session.id),
				data: InternalEventData::UserSession {
					after: user_session,
					data: StoredEventUserSessionData::Delete,
				},
				timestamp: chrono::Utc::now(),
			})?;
		}

		Ok(())
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

pub async fn parse_session(global: &Arc<Global>, ip: std::net::IpAddr, token: &str) -> Result<Option<Session>, ApiError> {
	let token = token.trim_matches('\"');

	let Some(jwt) = AuthJwtPayload::verify(global, token) else {
		return Ok(None);
	};

	let Some(session) = global
		.user_session_by_id_loader
		.load(jwt.session_id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load session"))?
	else {
		return Ok(None);
	};

	if session.expires_at < chrono::Utc::now() {
		return Ok(None);
	}

	global.user_session_updater_batcher.load(session.id).await.ok();

	let Some(user) = global
		.user_loader
		.load(global, session.user_id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
	else {
		return Ok(None);
	};

	Ok(Some(Session(
		Arc::new(AuthState::Authenticated {
			session,
			user: Box::new(user),
		}),
		ip,
	)))
}

/// If the token expires or is invalid and they are trying to login we should
/// just silently ignore the auth failure and let the request continue as if
/// they are not logged in. This is to prevent the client from being unable to
/// login if the token is expired.
fn request_is_v3_auth<B>(req: &Request<B>) -> bool {
	(req.uri().path() == "/v3/auth" || req.uri().path().starts_with("/v3/auth/")) && req.method() == hyper::Method::GET
}

impl<S> SessionMiddlewareService<S> {
	#[tracing::instrument(skip_all, name = "SessionMiddleware", fields(user_id, auth_failed))]
	async fn modify_request<B>(&self, req: &mut Request<B>) -> Result<(Option<RateLimitResponse>, bool), ApiError> {
		let cookies = req.extensions().get::<Cookies>().expect("cookies not found");
		let ip = *req.extensions().get::<std::net::IpAddr>().expect("ip not found");
		let cookie = cookies.get(AUTH_COOKIE);
		let ignore_auth_failure = req.headers().get("x-ignore-auth-failure").is_some_and(|v| v == "true");

		let mut session = None;
		let mut auth_failed = false;

		if let Some(token) = cookie.as_ref().map(|c| c.value()).or_else(|| {
			req.headers()
				.get(header::AUTHORIZATION)
				.and_then(|v| v.to_str().ok())
				.map(|s| s.trim_start_matches("Bearer "))
		}) {
			session = parse_session(&self.global, ip, token).await?;
			if session.is_none() {
				cookies.remove(&self.global, AUTH_COOKIE);
				auth_failed = true;
				if !ignore_auth_failure && !request_is_v3_auth(req) {
					return Err(ApiError::unauthorized(ApiErrorCode::LoginRequired, "invalid session"));
				}
			}
		}

		tracing::Span::current().record("user_id", session.as_ref().and_then(|s| s.user_id()).map(|u| u.to_string()));
		tracing::Span::current().record("auth_failed", auth_failed);

		let session = if let Some(session) = session {
			session
		} else {
			// Will load only the default permissions
			let default = self
				.global
				.user_loader
				.computed_loader
				.load(UserId::nil())
				.await
				.map_err(|()| {
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load default permissions")
				})?
				.ok_or_else(|| {
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load default permissions")
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

		Ok((ratelimit, auth_failed))
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
			let (rate_limit_resp, auth_failed) = match this.modify_request(&mut req).await {
				Ok(rate_limit_resp) => rate_limit_resp,
				Err(err) => return Ok(err.into_response()),
			};

			let mut resp = this.inner.call(req).await?.into_response();

			if let Some(rate_limit_resp) = rate_limit_resp {
				resp.headers_mut().extend(rate_limit_resp.header_map());
			}

			if auth_failed {
				resp.headers_mut().insert("x-auth-failure", "true".parse().unwrap());
			}

			Ok(resp)
		})
	}
}
