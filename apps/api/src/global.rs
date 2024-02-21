use anyhow::Context as _;
use async_nats::connection::State;
use scuffle_utils::context::Context;

use crate::config::Config;
use crate::metrics;

pub struct Global {
	ctx: Context,
	nats: async_nats::Client,
	config: Config,
	http_client: reqwest::Client,
	metrics: metrics::Metrics,
}

impl Global {
	pub async fn new(ctx: Context, config: Config) -> anyhow::Result<Self> {
		let nats = async_nats::connect(&config.nats.url).await.context("nats connect")?;

		Ok(Self {
			metrics: metrics::Metrics::new(
				config
					.monitoring
					.labels
					.iter()
					.map(|x| (x.key.clone(), x.value.clone()))
					.collect(),
			),
			ctx,
			nats,
			config,
			http_client: reqwest::Client::new(),
		})
	}

	/// The global context.
	pub fn ctx(&self) -> &Context {
		&self.ctx
	}

	/// The NATS client.
	pub fn nats(&self) -> &async_nats::Client {
		&self.nats
	}

	/// The configuration.
	pub fn config(&self) -> &Config {
		&self.config
	}

	/// Global HTTP client.
	pub fn http_client(&self) -> &reqwest::Client {
		&self.http_client
	}

	/// Global metrics.
	pub fn metrics(&self) -> &metrics::Metrics {
		&self.metrics
	}
}

impl shared::metrics::MetricsProvider for Global {
	fn ctx(&self) -> &scuffle_utils::context::Context {
		&self.ctx
	}

	fn bind(&self) -> std::net::SocketAddr {
		self.config.monitoring.bind
	}

	fn registry(&self) -> &prometheus_client::registry::Registry {
		self.metrics.registry()
	}

	fn pre_hook(&self) {
		self.metrics.observe_memory()
	}
}

impl shared::health::HealthProvider for Global {
	fn bind(&self) -> std::net::SocketAddr {
		self.config.health.bind
	}

	fn ctx(&self) -> &scuffle_utils::context::Context {
		&self.ctx
	}

	fn healthy(&self, _path: &str) -> bool {
		matches!(self.nats.connection_state(), State::Connected)
	}
}
