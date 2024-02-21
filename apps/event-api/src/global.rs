use std::sync::Arc;

use anyhow::Context as _;
use scuffle_utils::context::Context;

use crate::config::Config;
use crate::{metrics, subscription};

pub struct Global {
	ctx: Context,
	nats: async_nats::Client,
	config: Config,
	subscription_manager: subscription::SubscriptionManager,
	active_connections: Arc<std::sync::atomic::AtomicUsize>,
	http_client: reqwest::Client,
	metrics: metrics::Metrics,
}

/// An atomic ticket.
/// This is used to increment and decrement the number of active connections.
/// When the ticket is dropped, the number of active connections is decremented.
pub struct AtomicTicket(Arc<std::sync::atomic::AtomicUsize>);

impl AtomicTicket {
	/// Create a new atomic ticket.
	/// This will increment the number of active connections and return the
	/// current value.
	fn new(atomic: Arc<std::sync::atomic::AtomicUsize>) -> (Self, usize) {
		let x = atomic.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
		(Self(atomic), x)
	}
}

/// Decrement the number of active connections.
/// This is done when the ticket is dropped.
impl Drop for AtomicTicket {
	fn drop(&mut self) {
		self.0.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
	}
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
			subscription_manager: Default::default(),
			http_client: reqwest::Client::new(),
			active_connections: Default::default(),
		})
	}

	/// The number of active connections.
	pub fn active_connections(&self) -> usize {
		self.active_connections.load(std::sync::atomic::Ordering::Relaxed)
	}

	/// Increment the number of active connections.
	pub fn inc_active_connections(&self) -> (AtomicTicket, usize) {
		AtomicTicket::new(self.active_connections.clone())
	}

	/// The subscription manager.
	pub fn subscription_manager(&self) -> &subscription::SubscriptionManager {
		&self.subscription_manager
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

	fn healthy(&self, path: &str) -> bool {
		(match path {
			"/capacity" => {
				if let Some(limit) = self.config.api.connection_target.or(self.config.api.connection_limit) {
					self.active_connections() < limit
				} else {
					true
				}
			}
			_ => true,
		}) && matches!(self.nats.connection_state(), async_nats::connection::State::Connected)
	}
}
