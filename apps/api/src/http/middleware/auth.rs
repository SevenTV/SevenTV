use std::sync::Arc;

use hyper::StatusCode;
use mongodb::bson::doc;
use scuffle_utils::http::ext::{OptionExt, ResultExt};
use scuffle_utils::http::router::middleware::{Middleware, NextFn};
use scuffle_utils::http::RouteError;
use shared::database::{Collection, UserSession};

use super::AUTH_COOKIE;
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
		let global: Arc<Global> = req.get_global()?;

		let cookies = req.get_cookies()?;
		if let Some(cookie) = cookies.get(AUTH_COOKIE) {
			let jwt = AuthJwtPayload::verify(&global, cookie.value())
				.map_err_route((StatusCode::UNAUTHORIZED, "invalid auth token"))?;

			let session = UserSession::collection(global.db())
				.find_one_and_update(
					doc! {
						"_id": jwt.session_id,
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
				.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to update session"))?;

			if let Some(session) = session {
				req.extensions_mut().insert(session);
			} else {
				cookies.remove(AUTH_COOKIE);

				return Err((StatusCode::UNAUTHORIZED, "expired/invalid session").into());
			}
		}

		next(req).await
	}
}
