//! Cookie middleware

use std::sync::Arc;

use cookie::{Cookie, CookieBuilder, CookieJar};
use hyper::header::{HeaderValue, ToStrError};
use hyper::StatusCode;
use scuffle_utils::http::ext::ResultExt;
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::middleware::{Middleware, NextFn};
use scuffle_utils::http::RouteError;
use tokio::sync::RwLock;

use crate::global::Global;
use crate::http::error::ApiError;

pub const AUTH_COOKIE: &str = "seventv-auth";

pub type Cookies = Arc<RwLock<CookieJar>>;

pub fn new_cookie<'c, C: Into<Cookie<'c>>>(global: &Arc<Global>, base: C) -> CookieBuilder<'c> {
	Cookie::build(base)
		.http_only(true)
		.domain(format!(".{}", global.config().api.domain))
		.path("/")
		.secure(true)
		.same_site(cookie::SameSite::None)
}

#[derive(Default)]
pub struct CookieMiddleware;

#[async_trait::async_trait]
impl<I: Send + 'static, O: Send + 'static> Middleware<I, O, RouteError<ApiError>> for CookieMiddleware {
	async fn handle(
		&self,
		mut req: hyper::Request<I>,
		next: NextFn<I, O, RouteError<ApiError>>,
	) -> Result<hyper::Response<O>, RouteError<ApiError>> {
		let jar = req
			.headers()
			.get_all(hyper::header::COOKIE)
			.iter()
			.try_fold(CookieJar::new(), |mut jar, h| {
				for c in Cookie::split_parse_encoded(h.to_str()?) {
					match c {
						Ok(cookie) => jar.add_original(cookie.into_owned()),
						Err(e) => tracing::debug!("failed to parse a cookie {}", e),
					}
				}
				Ok::<CookieJar, ToStrError>(jar)
			})
			.map_ignore_err_route((StatusCode::BAD_REQUEST, "invalid cookie header"))?;
		// Using a RwLock here feels a little weird but I didn't find a better solution
		// to keep a reference to the jar longer than the ownership of the request
		let jar = Arc::new(RwLock::new(jar));
		req.extensions_mut().insert(jar.clone());

		let mut res = next(req).await?;

		for cookie in jar.read().await.delta() {
			res.headers_mut().append(
				hyper::header::SET_COOKIE,
				HeaderValue::from_str(&cookie.encoded().to_string())
					.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to encode cookie"))?,
			);
		}
		Ok(res)
	}

	fn extend(&self, builder: RouterBuilder<I, O, RouteError<ApiError>>) -> RouterBuilder<I, O, RouteError<ApiError>> {
		builder
	}
}
