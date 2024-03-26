use std::sync::Arc;

use prometheus_client::registry::Registry;
use shared::metrics::memory::MemoryMetrics;
use shared::metrics::{self, MetricsProvider};

use crate::global::Global;

mod labels;

pub type Metrics = metrics::Metrics<AllMetrics>;

pub fn new(mut labels: Vec<(String, String)>) -> Metrics {
	labels.push(("rust_app".to_string(), env!("CARGO_PKG_NAME").to_string()));
	metrics::Metrics::new(labels, AllMetrics::default())
}

pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	shared::metrics::run(global.ctx(), &global.config().metrics.http, global.metrics().clone()).await
}

#[derive(Default)]
pub struct AllMetrics {
	pub api: ApiMetrics,
	pub memory: MemoryMetrics,
}

impl MetricsProvider for AllMetrics {
	fn register(&mut self, registry: &mut Registry) {
		self.api.register(registry);
		self.memory.register(registry);
	}

	fn pre_encode(&self) {
		self.memory.pre_encode();
		self.api.pre_encode();
	}
}

impl std::ops::Deref for AllMetrics {
	type Target = ApiMetrics;

	fn deref(&self) -> &Self::Target {
		&self.api
	}
}

#[derive(Default)]
pub struct ApiMetrics {}

impl MetricsProvider for ApiMetrics {
	fn register(&mut self, _registry: &mut Registry) {}

	fn pre_encode(&self) {}
}
