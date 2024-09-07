use std::future::Future;

use anyhow::Context as _;
use scuffle_foundations::telemetry::server::HealthCheck;

use crate::config::Config;

pub struct Global {
	pub nats: async_nats::Client,
	pub jetstream: async_nats::jetstream::Context,
	pub database: mongodb::Database,
	pub config: Config,
}

impl Global {
	pub async fn new(config: Config) -> anyhow::Result<Self> {
		let (nats, jetstream) = shared::nats::setup_nats("event-api", &config.nats)
			.await
			.context("nats connect")?;

		let database = mongodb::Client::with_uri_str(&config.database.uri)
			.await
			.context("mongo connect")?
			.default_database()
			.ok_or_else(|| anyhow::anyhow!("no default database"))?;

		Ok(Self {
			nats,
			config,
			jetstream,
			database,
		})
	}
}

impl HealthCheck for Global {
	fn check(&self) -> std::pin::Pin<Box<dyn Future<Output = bool> + Send + '_>> {
		Box::pin(async {
			tracing::info!("running health check");

			if !matches!(self.nats.connection_state(), async_nats::connection::State::Connected) {
				tracing::error!("nats not connected");
				return false;
			}

			if self.database.run_command(bson::doc! { "ping": 1 }).await.is_err() {
				tracing::error!("mongo not connected");
				return false;
			}

			true
		})
	}
}
