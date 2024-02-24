use std::sync::Arc;

use anyhow::Context as _;
use scuffle_utils::context::Context;

use crate::config::Config;
use crate::metrics;

pub struct Global {
	ctx: Context,
	nats: async_nats::Client,
	config: Config,
	metrics: Arc<metrics::Metrics>,
}

impl Global {
	pub async fn new(ctx: Context, config: Config) -> anyhow::Result<Self> {
		let nats = async_nats::connect(&config.nats.url).await.context("nats connect")?;

		Ok(Self {
			metrics: Arc::new(metrics::new(
				config
					.metrics
					.labels
					.iter()
					.map(|x| (x.key.clone(), x.value.clone()))
					.collect(),
			)),
			ctx,
			nats,
			config,
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

	/// Global metrics.
	pub fn metrics(&self) -> &Arc<metrics::Metrics> {
		&self.metrics
	}
}
