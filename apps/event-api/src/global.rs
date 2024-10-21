use std::sync::Arc;

use anyhow::Context as _;
use scuffle_foundations::telemetry::server::HealthCheck;

use crate::config::Config;
use crate::subscription;

pub struct Global {
	pub nats: async_nats::Client,
	pub config: Config,
	pub subscription_manager: subscription::SubscriptionManager,
	active_connections: Arc<std::sync::atomic::AtomicUsize>,
	pub http_client: reqwest::Client,
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
	pub async fn new(config: Config) -> anyhow::Result<Self> {
		let (nats, _) = shared::nats::setup_nats("event-api", &config.nats)
			.await
			.context("nats connect")?;

		Ok(Self {
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
}

impl HealthCheck for Global {
	fn check(&self) -> std::pin::Pin<Box<dyn futures::Future<Output = bool> + Send + '_>> {
		Box::pin(async {
			tracing::debug!("running health check");

			if !matches!(self.nats.connection_state(), async_nats::connection::State::Connected) {
				tracing::error!("nats not connected");
				return false;
			}

			true
		})
	}
}
