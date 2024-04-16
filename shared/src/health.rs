use std::sync::Arc;
use std::time::Duration;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response, StatusCode};
use scuffle_utils::context::Context;
use scuffle_utils::prelude::FutureTimeout;

use crate::config::Http;

#[derive(Debug, thiserror::Error)]
pub enum HealthError {
	#[error("timeout")]
	Timeout,
	#[error("error")]
	Error(#[from] anyhow::Error),
}

pub trait HealthCheck: Send + Sync + 'static {
	fn is_healthy(&self, path: &str) -> impl std::future::Future<Output = anyhow::Result<bool>> + Send;
}

impl<F, Fut> HealthCheck for F
where
	F: Send + Sync + 'static,
	F: Fn(&str) -> Fut,
	Fut: std::future::Future<Output = anyhow::Result<bool>> + Send + 'static,
{
	fn is_healthy(&self, path: &str) -> impl std::future::Future<Output = anyhow::Result<bool>> + Send {
		(self)(path)
	}
}

#[tracing::instrument(name = "health", level = "info", skip(ctx, config, health_check))]
pub async fn run(ctx: &Context, config: &Http, health_check: impl HealthCheck) -> anyhow::Result<()> {
	let health_check = Arc::new(health_check);

	let handler = move |req: Request<Incoming>| {
		let health_check = health_check.clone();

		async move {
			let path = req.uri().path();

			let status = health_check.is_healthy(path).timeout(Duration::from_secs(2)).await??;

			let body = if status {
				Bytes::from_static(b"OK")
			} else {
				Bytes::from_static(b"FAIL")
			};

			anyhow::Ok(Response::builder().status(StatusCode::OK).body(Full::new(body)).unwrap())
		}
	};

	crate::http::run(ctx, config, handler, |_| true).await
}
