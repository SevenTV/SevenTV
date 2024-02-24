use std::sync::Arc;

use anyhow::Context as _;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, StatusCode};
use scuffle_utils::context::Context;

use crate::config::Http;

pub mod http;
pub mod memory;

pub const DEFAULT_HISTOGRAM_BUCKETS: &[f64] = &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

pub trait MetricsProvider: Send + Sync + 'static {
	fn pre_encode(&self) {}
	fn register(&mut self, registry: &mut prometheus_client::registry::Registry);
}

pub struct Metrics<T: MetricsProvider> {
	inner: T,
	registry: prometheus_client::registry::Registry,
}

impl<T: MetricsProvider> Metrics<T> {
	pub fn new(labels: Vec<(String, String)>, mut inner: T) -> Self {
		let labels = labels.into_iter().map(|(k, v)| (k.into(), v.into()));
		let mut registry = prometheus_client::registry::Registry::with_prefix_and_labels("7tv", labels);

		inner.register(&mut registry);

		Self { inner, registry }
	}

	fn encode(&self) -> Result<String, std::fmt::Error> {
		self.inner.pre_encode();
		let mut body = String::new();
		prometheus_client::encoding::text::encode(&mut body, &self.registry)?;
		Ok(body)
	}
}

impl<T: MetricsProvider> std::ops::Deref for Metrics<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[tracing::instrument(name = "metrics", level = "info", skip(ctx, config, metrics))]
pub async fn run(ctx: &Context, config: &Http, metrics: Arc<Metrics<impl MetricsProvider>>) -> anyhow::Result<()> {
	let handler = move |_: Request<Incoming>| {
		let metrics = metrics.clone();

		async move {
			let body = metrics.encode().context("encode metrics")?;
			anyhow::Ok({
				hyper::Response::builder()
					.header(hyper::header::CONTENT_TYPE, "text/plain")
					.status(StatusCode::OK)
					.body(Full::new(Bytes::from(body)))
					.context("build response")?
			})
		}
	};

	crate::http::run(ctx, config, handler, |_| true).await
}
