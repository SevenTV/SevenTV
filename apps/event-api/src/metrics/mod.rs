use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use memory_stats::memory_stats;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;
use scuffle_utils::prelude::FutureTimeout;
use tokio::net::TcpSocket;

use self::labels::{
	ClientClose, Command, ConnectionDuration, CurrentConnection, Labels, Memory, NatsEvent, TotalSubscription,
	UniqueSubscriptions,
};
use crate::global::Global;
use crate::message::types::Opcode;

mod labels;

pub struct Metrics {
	registry: Registry,
	connection_duration_seconds: Family<Labels<ConnectionDuration>, Histogram>,
	current_connections: Family<Labels<CurrentConnection>, Gauge>,
	unique_subscriptions: Family<Labels<UniqueSubscriptions>, Gauge>,
	total_subscriptions: Family<Labels<TotalSubscription>, Gauge>,
	nats_events: Family<Labels<NatsEvent>, Counter>,
	commands: Family<Labels<Command>, Counter>,
	client_closes: Family<Labels<ClientClose>, Counter>,
	memory: Family<Labels<Memory>, Gauge>,
	labels: Labels<()>,
}

const DEFAULT_HISTOGRAM_BUCKETS: &[f64] = &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

impl Metrics {
	pub fn new(mut labels: Vec<(String, String)>) -> Self {
		let mut registry = Registry::default();

		let connection_duration_seconds =
			Family::<_, _>::new_with_constructor(|| Histogram::new(DEFAULT_HISTOGRAM_BUCKETS.iter().copied()));
		let current_connections = Family::default();
		let unique_subscriptions = Family::default();
		let total_subscriptions = Family::default();
		let nats_events = Family::default();
		let commands = Family::default();
		let client_closes = Family::default();
		let memory = Family::default();

		registry.register(
			"eventapi_connection_duration_seconds",
			"The number of seconds used on connections",
			connection_duration_seconds.clone(),
		);

		registry.register(
			"eventapi_current_connections",
			"The current number of connections",
			current_connections.clone(),
		);

		registry.register(
			"eventapi_unique_subscriptions",
			"The number of unique subscriptions",
			unique_subscriptions.clone(),
		);

		registry.register(
			"eventapi_total_subscriptions",
			"The number of total subscriptions",
			total_subscriptions.clone(),
		);

		registry.register("eventapi_nats_events", "The number of NATs events", nats_events.clone());

		registry.register("eventapi_commands", "The number of commands issued", commands.clone());

		registry.register("eventapi_client_closes", "The number of client closes", client_closes.clone());

		registry.register("eventapi_memory_bytes", "The amount of memory used", memory.clone());

		labels.push(("version".into(), env!("CARGO_PKG_VERSION").into()));

		Self {
			registry,
			connection_duration_seconds,
			current_connections,
			unique_subscriptions,
			total_subscriptions,
			nats_events,
			commands,
			client_closes,
			memory,
			labels: Labels::new(labels),
		}
	}

	/// Observe a client close code.
	pub fn observe_client_close(&self, code: &'static str) {
		self.client_closes
			.get_or_create(&self.labels.extend(ClientClose::new(code)))
			.inc();
	}

	/// Observe memory usage.
	pub fn observe_memory(&self) {
		self.memory
			.get_or_create(&self.labels.extend(Memory::ALLOCATED))
			.set(super::ALLOCATOR.allocated() as i64);
		self.memory
			.get_or_create(&self.labels.extend(Memory::REMAINING))
			.set(super::ALLOCATOR.remaining() as i64);

		if let Some(usage) = memory_stats() {
			self.memory
				.get_or_create(&self.labels.extend(Memory::RESIDENT))
				.set(usage.physical_mem as i64);
			self.memory
				.get_or_create(&self.labels.extend(Memory::VIRTUAL))
				.set(usage.virtual_mem as i64);
		} else {
			tracing::warn!("failed to get memory stats");
		}
	}

	/// Observe a nats event miss.
	pub fn observe_nats_event_miss(&self) {
		self.nats_events.get_or_create(&self.labels.extend(NatsEvent::MISS)).inc();
	}

	/// Observe a nats event hit.
	pub fn observe_nats_event_hit(&self) {
		self.nats_events.get_or_create(&self.labels.extend(NatsEvent::HIT)).inc();
	}

	/// Observe how long a connection was open.
	pub fn observe_connection_duration_seconds(&self, duration: f64) {
		self.connection_duration_seconds
			.get_or_create(&self.labels.extend(ConnectionDuration::V3))
			.observe(duration);
	}

	/// Set the number of unique subscriptions.
	pub fn set_unique_subscriptions(&self, len: usize, cap: usize) {
		self.unique_subscriptions
			.get_or_create(&self.labels.extend(UniqueSubscriptions::LEN))
			.set(len as i64);
		self.unique_subscriptions
			.get_or_create(&self.labels.extend(UniqueSubscriptions::CAP))
			.set(cap as i64);
	}

	/// Observe a command from the server.
	pub fn observe_server_command(&self, code: Opcode) {
		self.commands.get_or_create(&self.labels.extend(Command::server(code))).inc();
	}

	/// Observe a command from the client.
	pub fn observe_client_command(&self, code: Opcode) {
		self.commands.get_or_create(&self.labels.extend(Command::client(code))).inc();
	}

	pub fn incr_total_subscriptions(&self) {
		self.total_subscriptions
			.get_or_create(&self.labels.extend(TotalSubscription::V3))
			.inc();
	}

	pub fn decr_total_subscriptions(&self) {
		self.total_subscriptions
			.get_or_create(&self.labels.extend(TotalSubscription::V3))
			.dec();
	}

	pub fn incr_current_event_streams(&self) {
		self.current_connections
			.get_or_create(&self.labels.extend(CurrentConnection::EVENT_STREAM))
			.inc();
	}

	pub fn decr_current_event_streams(&self) {
		self.current_connections
			.get_or_create(&self.labels.extend(CurrentConnection::EVENT_STREAM))
			.dec();
	}

	pub fn incr_current_websocket_connections(&self) {
		self.current_connections
			.get_or_create(&self.labels.extend(CurrentConnection::WEBSOCKET))
			.inc();
	}

	pub fn decr_current_websocket_connections(&self) {
		self.current_connections
			.get_or_create(&self.labels.extend(CurrentConnection::WEBSOCKET))
			.dec();
	}

	pub fn registry(&self) -> &Registry {
		&self.registry
	}
}

pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let config = global.config();
	tracing::info!("[metrics] listening on http://{}", config.monitoring.bind);
	let socket = if config.monitoring.bind.is_ipv6() {
		TcpSocket::new_v6()?
	} else {
		TcpSocket::new_v4()?
	};

	socket.set_reuseaddr(true).context("socket reuseaddr")?;
	socket.set_reuseport(true).context("socket reuseport")?;
	socket.bind(config.monitoring.bind).context("socket bind")?;
	let listener = socket.listen(16)?;

	loop {
		tokio::select! {
			_ = global.ctx().done() => {
				return Ok(());
			},
			r = listener.accept() => {
				let (socket, _) = r?;

				let registry = global.metrics().registry();
				let global = &global;

				let service = service_fn(move |_| async {
					let mut body = String::new();

					global.metrics().observe_memory();

					prometheus_client::encoding::text::encode(&mut body, registry).context("encode prometheus metrics")?;

					Ok::<_, anyhow::Error>({
						hyper::Response::builder()
							.header(hyper::header::CONTENT_TYPE, "text/plain")
							.status(StatusCode::OK)
							.body(Full::new(Bytes::from(body)))
							.context("build response")?
					})
				});

				let http = http1::Builder::new();

				http.serve_connection(
					TokioIo::new(socket),
					service,
				).timeout(Duration::from_secs(2)).await.ok();
			},
		}
	}
}
