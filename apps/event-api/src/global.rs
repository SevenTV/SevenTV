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
	metrics: Arc<metrics::Metrics>,
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
		let (nats, _) = shared::nats::setup_nats("event-api", &config.nats)
			.await
			.context("nats connect")?;

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
	pub fn metrics(&self) -> &Arc<metrics::Metrics> {
		&self.metrics
	}
}
