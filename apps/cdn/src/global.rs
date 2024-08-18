use scuffle_foundations::telemetry::server::HealthCheck;

use crate::{cache, config::Config};

pub struct Global {
	pub config: Config,
	pub cache: cache::Cache,
}

impl Global {
	pub async fn new(config: Config) -> Self {
		Self {
			cache: cache::Cache::new(config.cdn.cache_capacity, &config.cdn.bucket),
			config,
		}
	}
}

impl HealthCheck for Global {
	fn check(&self) -> std::pin::Pin<Box<dyn futures::Future<Output = bool> + Send + '_>> {
		Box::pin(async {
			tracing::info!("running health check");

			true
		})
	}
}
