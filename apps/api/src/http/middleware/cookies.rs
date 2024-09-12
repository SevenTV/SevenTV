//! Cookie middleware

use std::sync::{Arc, Mutex};

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use cookie::{Cookie, CookieBuilder, CookieJar};
use hyper::header::HeaderValue;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

#[derive(Clone)]
pub struct Cookies(Arc<Mutex<CookieJar>>);

impl Cookies {
	pub fn delta(&self) -> Vec<Cookie<'static>> {
		self.0.lock().unwrap().delta().cloned().collect()
	}

	pub fn add(&self, cookie: impl Into<Cookie<'static>>) {
		self.0.lock().unwrap().add(cookie);
	}

	pub fn get(&self, name: &str) -> Option<Cookie<'static>> {
		self.0.lock().unwrap().get(name).cloned()
	}

	pub fn remove(&self, global: &Arc<Global>, name: impl Into<Cookie<'static>>) {
		let mut cookie: Cookie = name.into();
		cookie.set_domain(format!(".{}", global.config.api.domain));
		cookie.set_path("/");
		self.0.lock().unwrap().remove(cookie);
	}
}

pub fn new_cookie<'c, C: Into<Cookie<'c>>>(global: &Arc<Global>, base: C) -> CookieBuilder<'c> {
	Cookie::build(base)
		.http_only(true)
		.domain(format!(".{}", global.config.api.domain))
		.path("/")
		.secure(true)
		.same_site(cookie::SameSite::None)
}

#[derive(Default, Clone, Debug, Copy)]
pub struct CookieMiddleware;

impl<S> tower::Layer<S> for CookieMiddleware {
	type Service = CookieMiddlewareService<S>;

	fn layer(&self, inner: S) -> Self::Service {
		CookieMiddlewareService { inner }
	}
}

#[derive(Clone)]
pub struct CookieMiddlewareService<S> {
	inner: S,
}

#[pin_project::pin_project(project = CookieFutureProj)]
pub enum CookieFuture<F> {
	EarlyError(Option<ApiError>),
	Inner(#[pin] F, Cookies),
}

impl<F, E> std::future::Future for CookieFuture<F>
where
	F: std::future::Future<Output = Result<Response, E>>,
{
	type Output = Result<Response, E>;

	fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
		match self.project() {
			CookieFutureProj::EarlyError(err) => {
				std::task::Poll::Ready(Ok(err.take().expect("error already taken").into_response()))
			}
			CookieFutureProj::Inner(fut, cookies) => {
				let mut resp = match fut.poll(cx) {
					std::task::Poll::Ready(Ok(res)) => res,
					std::task::Poll::Ready(Err(err)) => return std::task::Poll::Ready(Err(err)),
					std::task::Poll::Pending => return std::task::Poll::Pending,
				};

				for cookie in cookies.delta() {
					resp.headers_mut().append(
						hyper::header::SET_COOKIE,
						HeaderValue::from_str(&cookie.encoded().to_string()).unwrap(),
					);
				}

				std::task::Poll::Ready(Ok(resp))
			}
		}
	}
}

impl<S, B> tower::Service<Request<B>> for CookieMiddlewareService<S>
where
	S: tower::Service<Request<B>, Response = Response> + Clone + Send,
	S::Error: Send,
	S::Future: Send,
	B: Send,
{
	type Error = S::Error;
	type Future = CookieFuture<S::Future>;
	type Response = S::Response;

	fn call(&mut self, mut req: Request<B>) -> Self::Future {
		let jar = match req
			.headers()
			.get_all(hyper::header::COOKIE)
			.iter()
			.try_fold(CookieJar::new(), |mut jar, h| {
				for c in Cookie::split_parse_encoded(
					h.to_str()
						.map_err(|_| ApiError::bad_request(ApiErrorCode::Cookie, "cookie header is not a valid string"))?,
				) {
					match c {
						Ok(cookie) => jar.add_original(cookie.into_owned()),
						Err(_) => return Err(ApiError::bad_request(ApiErrorCode::Cookie, "invalid cookie header")),
					}
				}
				Ok(jar)
			}) {
			Ok(jar) => jar,
			Err(err) => return CookieFuture::EarlyError(Some(err)),
		};

		let jar = Cookies(Arc::new(Mutex::new(jar)));
		req.extensions_mut().insert(jar.clone());

		CookieFuture::Inner(self.inner.call(req), jar)
	}

	fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx).map_err(Into::into)
	}
}
