use scuffle_foundations::telemetry::server::HealthCheck;

use crate::config::Config;

pub struct Global {
    pub config: Config,
    pub http_client: reqwest::Client,
}

impl Global {
    pub async fn new(config: Config) -> Self {
		Self {
			config,
            http_client: reqwest::Client::new(),
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
