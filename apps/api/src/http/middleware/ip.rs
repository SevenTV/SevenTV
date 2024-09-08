//! IP middleware

use std::sync::Arc;

use axum::extract::Request;
use axum::response::{IntoResponse, Response};

use crate::global::Global;
use crate::http::error::ApiError;

#[derive(Clone)]
pub struct IpMiddleware(Arc<Global>);

impl IpMiddleware {
	pub fn new(global: Arc<Global>) -> Self {
		Self(global)
	}
}

impl<S> tower::Layer<S> for IpMiddleware {
	type Service = IpMiddlewareService<S>;

	fn layer(&self, inner: S) -> Self::Service {
		IpMiddlewareService {
			inner,
			global: self.0.clone(),
		}
	}
}

#[derive(Clone)]
pub struct IpMiddlewareService<S> {
	inner: S,
	global: Arc<Global>,
}

impl<S> IpMiddlewareService<S> {
	fn modify<B>(&mut self, req: &mut Request<B>) -> Result<(), ApiError> {
		let connecting_ip = req
			.extensions()
			.get::<std::net::IpAddr>()
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let trusted_proxies = &self.global.config.api.incoming_request.trusteded_proxies;

		if trusted_proxies.is_empty() {
			return Ok(());
		}

		// If the IP is not a trusted proxy, we should return a 403.
		if trusted_proxies.iter().all(|net| !net.contains(connecting_ip)) {
			return Err(ApiError::FORBIDDEN);
		}

		let Some(header) = &self.global.config.api.incoming_request.ip_header else {
			tracing::warn!("missing ip_header but trusted proxies are enabled");
			return Ok(());
		};

		let ips = req.headers().get(header).ok_or(ApiError::FORBIDDEN)?;
		let ips = ips.to_str().map_err(|_| ApiError::BAD_REQUEST)?;
		let ips = ips.split(',').map(|ip| ip.trim());
		let ips = ips
			.map(|ip| ip.parse::<std::net::IpAddr>())
			.collect::<Result<Vec<_>, _>>()
			.map_err(|_| ApiError::BAD_REQUEST)?;

		for ip in ips.into_iter().rev() {
			if trusted_proxies.iter().all(|net| !net.contains(&ip)) {
				req.extensions_mut().insert(ip);
				return Ok(());
			}
		}

		tracing::warn!("no ips found in header that are not trusted proxies");

		Ok(())
	}
}

impl<S, B> tower::Service<Request<B>> for IpMiddlewareService<S>
where
	S: tower::Service<Request<B>, Response = Response> + Clone + Send,
	S::Error: Send,
	S::Future: Send,
	B: Send,
{
	type Error = S::Error;
	type Future = futures::future::Either<futures::future::Ready<Result<Self::Response, Self::Error>>, S::Future>;
	type Response = S::Response;

	fn call(&mut self, mut req: Request<B>) -> Self::Future {
		match self.modify(&mut req) {
			Ok(_) => futures::future::Either::Right(self.inner.call(req)),
			Err(e) => futures::future::Either::Left(futures::future::ready(Ok(e.into_response()))),
		}
	}

	fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx).map_err(Into::into)
	}
}
