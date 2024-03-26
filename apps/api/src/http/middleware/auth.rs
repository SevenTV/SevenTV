use std::sync::Arc;

use hyper::StatusCode;
use scuffle_utils::http::ext::{OptionExt, ResultExt};
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::middleware::{Middleware, NextFn};
use scuffle_utils::http::RouteError;

use super::AUTH_COOKIE;
use crate::database::UserSession;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::RequestGlobalExt;
use crate::jwt::{AuthJwtPayload, JwtState};

#[derive(Default)]
pub struct AuthMiddleware;

#[async_trait::async_trait]
impl<I: Send + 'static, O: Send + 'static> Middleware<I, O, RouteError<ApiError>> for AuthMiddleware {
	async fn handle(
		&self,
		mut req: hyper::Request<I>,
		next: NextFn<I, O, RouteError<ApiError>>,
	) -> Result<hyper::Response<O>, RouteError<ApiError>> {
		{
			let global: Arc<Global> = req.get_global()?;
			let cookies = req.get_cookies()?;
			let mut cookies = cookies.write().await;
			if let Some(cookie) = cookies.get(AUTH_COOKIE) {
				let jwt = AuthJwtPayload::verify(&global, cookie.value())
					.map_err_route((StatusCode::UNAUTHORIZED, "invalid auth token"))?;
				let session: Option<UserSession> = scuffle_utils::database::query(
					"UPDATE user_sessions SET last_used_at = NOW() WHERE id = $1 AND expires_at > NOW() RETURNING *",
				)
				.bind(jwt.session_id)
				.build_query_as()
				.fetch_optional(&global.db())
				.await
				.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch session"))?;
				if let Some(session) = session {
					req.extensions_mut().insert(session);
				} else {
					cookies.remove(AUTH_COOKIE);
					return Err((StatusCode::UNAUTHORIZED, "expired/invalid session").into());
				}
			}
		}
		next(req).await
	}

	fn extend(&self, builder: RouterBuilder<I, O, RouteError<ApiError>>) -> RouterBuilder<I, O, RouteError<ApiError>> {
		builder
	}
}
