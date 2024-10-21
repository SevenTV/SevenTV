//! IP middleware

use std::sync::Arc;

use axum::extract::Request;
use axum::response::{IntoResponse, Response};

#[derive(Clone)]
pub struct IpMiddleware(Arc<crate::config::IncomingRequestConfig>);

impl IpMiddleware {
	pub fn new(config: crate::config::IncomingRequestConfig) -> Self {
		Self(Arc::new(config))
	}
}

impl<S> tower::Layer<S> for IpMiddleware {
	type Service = IpMiddlewareService<S>;

	fn layer(&self, inner: S) -> Self::Service {
		IpMiddlewareService {
			inner,
			config: self.0.clone(),
		}
	}
}

#[derive(Clone)]
pub struct IpMiddlewareService<S> {
	inner: S,
	config: Arc<crate::config::IncomingRequestConfig>,
}

impl<S> IpMiddlewareService<S> {
	fn modify<B>(&mut self, req: &mut Request<B>) -> Result<(), axum::response::Response> {
		let connecting_ip = req
			.extensions()
			.get::<std::net::IpAddr>()
			.ok_or_else(|| {
				axum::response::Response::builder()
					.status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
					.body("missing connecting ip address".into())
					.unwrap()
			})?
			.to_canonical();

		let trusted_proxies = &self.config.trusted_proxies;
		let trusted_ranges = &self.config.trusted_ranges;

		if trusted_proxies.is_empty() || trusted_ranges.iter().any(|net| net.contains(&connecting_ip)) {
			return Ok(());
		}

		// If the IP is not a trusted proxy, we should return a 403.
		if trusted_proxies.iter().all(|net| !net.contains(&connecting_ip)) {
			return Err(axum::response::Response::builder()
				.status(axum::http::StatusCode::FORBIDDEN)
				.body("ip is not trusted".into())
				.unwrap());
		}

		let Some(header) = &self.config.ip_header else {
			return Ok(());
		};

		let ips = req
			.headers()
			.get(header)
			.ok_or_else(|| {
				axum::response::Response::builder()
					.status(axum::http::StatusCode::FORBIDDEN)
					.body("missing ip header".into())
					.unwrap()
			})?
			.to_str()
			.map_err(|_| {
				axum::response::Response::builder()
					.status(axum::http::StatusCode::BAD_REQUEST)
					.body("ip header not valid".into())
					.unwrap()
			})?
			.split(',')
			.map(|ip| ip.trim())
			.map(|ip| ip.parse::<std::net::IpAddr>())
			.collect::<Result<Vec<_>, _>>()
			.map_err(|_| {
				axum::response::Response::builder()
					.status(axum::http::StatusCode::BAD_REQUEST)
					.body("invalid ip header".into())
					.unwrap()
			})?;

		for ip in ips.into_iter().rev() {
			if trusted_proxies.iter().all(|net| !net.contains(&ip)) {
				req.extensions_mut().insert(ip);
				return Ok(());
			}
		}

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
