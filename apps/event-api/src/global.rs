use std::sync::Arc;

use anyhow::Context as _;
use scuffle_bootstrap_telemetry::opentelemetry;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::metrics::SdkMeterProvider;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::Resource;
use scuffle_metrics::opentelemetry::KeyValue;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

use crate::config::Config;
use crate::subscription;

pub struct Global {
	pub nats: async_nats::Client,
	pub config: Config,
	pub subscription_manager: subscription::SubscriptionManager,
	active_connections: Arc<std::sync::atomic::AtomicUsize>,
	prometheus_registry: scuffle_bootstrap_telemetry::prometheus::Registry,
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

impl scuffle_bootstrap::global::Global for Global {
	type Config = Config;

	fn pre_init() -> anyhow::Result<()> {
		rustls::crypto::aws_lc_rs::default_provider().install_default().ok();

		Ok(())
	}

	async fn init(config: Self::Config) -> anyhow::Result<Arc<Self>> {
		let prometheus_registry = scuffle_bootstrap_telemetry::prometheus::Registry::new();

		opentelemetry::global::set_meter_provider(
			SdkMeterProvider::builder()
				.with_resource(Resource::new(vec![KeyValue::new("service.name", env!("CARGO_BIN_NAME"))]))
				.with_reader(
					scuffle_metrics::prometheus::exporter()
						.with_registry(prometheus_registry.clone())
						.build()
						.context("prometheus metrics exporter")?,
				)
				.build(),
		);

		tracing_subscriber::registry()
			.with(
				tracing_subscriber::fmt::layer()
					.with_file(true)
					.with_line_number(true)
					.with_filter(
						EnvFilter::builder()
							.with_default_directive(LevelFilter::INFO.into())
							.parse_lossy(&config.level),
					),
			)
			.init();

		let (nats, _) = shared::nats::setup_nats("event-api", &config.nats)
			.await
			.context("nats connect")?;

		Ok(Arc::new(Self {
			nats,
			config,
			subscription_manager: Default::default(),
			active_connections: Default::default(),
			prometheus_registry,
		}))
	}
}

impl Global {
	/// The number of active connections.
	pub fn active_connections(&self) -> usize {
		self.active_connections.load(std::sync::atomic::Ordering::Relaxed)
	}

	/// Increment the number of active connections.
	pub fn inc_active_connections(&self) -> (AtomicTicket, usize) {
		AtomicTicket::new(self.active_connections.clone())
	}
}

impl scuffle_bootstrap_telemetry::TelemetryConfig for Global {
	fn bind_address(&self) -> Option<std::net::SocketAddr> {
		self.config.metrics_bind
	}

	fn prometheus_metrics_registry(&self) -> Option<&scuffle_bootstrap_telemetry::prometheus::Registry> {
		Some(&self.prometheus_registry)
	}

	async fn health_check(&self) -> Result<(), anyhow::Error> {
		tracing::debug!("running health check");

		if !matches!(self.nats.connection_state(), async_nats::connection::State::Connected) {
			anyhow::bail!("nats not connected");
		}

		Ok(())
	}
}

impl scuffle_bootstrap::signals::SignalSvcConfig for Global {
	async fn on_shutdown(self: &Arc<Self>) -> anyhow::Result<()> {
		tracing::info!("shutting down event-api");
		Ok(())
	}
}
