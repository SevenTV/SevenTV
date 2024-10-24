use anyhow::Context;
use scuffle_foundations::telemetry::server::HealthCheck;

use crate::cache;
use crate::config::Config;

pub struct Global {
	pub config: Config,
	pub cache: cache::Cache,
	pub jetstream: async_nats::jetstream::Context,
}

impl Global {
	pub async fn new(config: Config) -> anyhow::Result<Self> {
		let (_, jetstream) = shared::nats::setup_nats(&config.pod.name, &config.nats)
			.await
			.context("nats")?;

		Ok(Self {
			cache: cache::Cache::new(&config.cdn),
			config,
			jetstream,
		})
	}
}

impl HealthCheck for Global {
	fn check(&self) -> std::pin::Pin<Box<dyn futures::Future<Output = bool> + Send + '_>> {
		Box::pin(async {
			tracing::debug!("running health check");

			true
		})
	}
}
