use std::sync::Arc;

use axum::extract::Request;
use axum::response::Response;
use futures::future::BoxFuture;
use futures::FutureExt;
use hyper::{header, StatusCode};
use mongodb::bson::doc;
use shared::database::{Collection, UserId, UserSession};

use super::cookies::Cookies;
use crate::global::Global;
use crate::http::error::{map_result, ApiError, EitherApiError};
use crate::jwt::{AuthJwtPayload, JwtState};

pub const AUTH_COOKIE: &str = "seventv-auth";

#[derive(Clone)]
pub struct AuthMiddleware(Arc<Global>);

impl AuthMiddleware {
	pub fn new(global: Arc<Global>) -> Self {
		Self(global)
	}
}

impl<S> tower::Layer<S> for AuthMiddleware {
	type Service = AuthMiddlewareService<S>;

	fn layer(&self, inner: S) -> Self::Service {
		AuthMiddlewareService {
			inner,
			global: self.0.clone(),
		}
	}
}

#[derive(Clone)]
pub struct AuthMiddlewareService<S> {
	global: Arc<Global>,
	inner: S,
}

#[derive(Debug, Clone)]
pub enum AuthSession {
	/// The user session
	Session(UserSession),
	/// Old user sessions, only user id available
	Old(UserId),
}

impl AuthSession {
	pub fn user_id(&self) -> UserId {
		match self {
			Self::Session(session) => session.user_id,
			Self::Old(user_id) => *user_id,
		}
	}
}

impl<S> AuthMiddlewareService<S> {
	async fn serve<B>(mut self, mut req: Request<B>) -> Result<Response, EitherApiError<S::Error>>
	where
		S: tower::Service<Request<B>, Response = Response> + Clone + Send,
		S::Error: std::error::Error + Send,
		S::Future: Send,
		B: Send,
	{
		let cookies = req.extensions().get::<Cookies>().expect("cookies not found");
		let auth_cookie = cookies.get(AUTH_COOKIE);

		if let Some(token) = auth_cookie.as_ref().map(|c| c.value()).or_else(|| {
			req.headers()
				.get(header::AUTHORIZATION)
				.and_then(|v| v.to_str().ok())
				.map(|s| s.trim_start_matches("Bearer "))
		}) {
			let jwt = AuthJwtPayload::verify(&self.global, token).ok_or_else(|| {
				cookies.remove(AUTH_COOKIE);
				ApiError::UNAUTHORIZED
			})?;

			match jwt.session_id {
				Some(session_id) => {
					let session = UserSession::collection(self.global.db())
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

					req.extensions_mut().insert(AuthSession::Session(session));
				}
				// old session
				None => {
					req.extensions_mut().insert(AuthSession::Old(jwt.user_id));
				}
			}
		}

		self.inner.call(req).await.map_err(EitherApiError::Other)
	}
}

impl<S, B> tower::Service<Request<B>> for AuthMiddlewareService<S>
where
	S: tower::Service<Request<B>, Response = Response> + Clone + Send + 'static,
	S::Error: std::error::Error + Send + 'static,
	S::Future: Send + 'static,
	B: Send + 'static,
{
	type Error = S::Error;
	type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
	type Response = S::Response;

	fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx).map_err(Into::into)
	}

	fn call(&mut self, req: Request<B>) -> Self::Future {
		Box::pin(self.clone().serve(req).map(map_result))
	}
}
