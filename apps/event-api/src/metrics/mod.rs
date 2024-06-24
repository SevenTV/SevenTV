use std::sync::Arc;

use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;
use shared::event_api::types::Opcode;

use self::labels::{
	ClientClose, Command, ConnectionDuration, CurrentConnection, NatsEvent, TotalSubscription, UniqueSubscriptions,
};
use crate::global::Global;

mod labels;

pub type Metrics = metrics::Metrics<AllMetrics>;

pub fn new(mut labels: Vec<(String, String)>) -> Metrics {
	labels.push(("rust_app".to_string(), env!("CARGO_PKG_NAME").to_string()));
	metrics::Metrics::new(labels, AllMetrics::default())
}

pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	shared::metrics::run(global.ctx(), &global.config().metrics.http, global.metrics().clone()).await
}

#[derive(Default)]
pub struct AllMetrics {
	pub event_api: EventApiMetrics,
	pub memory: MemoryMetrics,
}

impl std::ops::Deref for AllMetrics {
	type Target = EventApiMetrics;

	fn deref(&self) -> &Self::Target {
		&self.event_api
	}
}

impl MetricsProvider for AllMetrics {
	fn register(&mut self, registry: &mut Registry) {
		self.event_api.register(registry);
		self.memory.register(registry);
	}

	fn pre_encode(&self) {
		self.memory.pre_encode();
		self.event_api.pre_encode();
	}
}

impl AsRef<EventApiMetrics> for AllMetrics {
	fn as_ref(&self) -> &EventApiMetrics {
		&self.event_api
	}
}

impl AsRef<MemoryMetrics> for AllMetrics {
	fn as_ref(&self) -> &MemoryMetrics {
		&self.memory
	}
}

pub struct EventApiMetrics {
	connection_duration_seconds: Family<ConnectionDuration, Histogram>,
	current_connections: Family<CurrentConnection, Gauge>,
	unique_subscriptions: Family<UniqueSubscriptions, Gauge>,
	total_subscriptions: Family<TotalSubscription, Gauge>,
	nats_events: Family<NatsEvent, Counter>,
	commands: Family<Command, Counter>,
	client_closes: Family<ClientClose, Counter>,
}

impl Default for EventApiMetrics {
	fn default() -> Self {
		let connection_duration_seconds =
			Family::<_, _>::new_with_constructor(|| Histogram::new(DEFAULT_HISTOGRAM_BUCKETS.iter().copied()));
		let current_connections = Family::default();
		let unique_subscriptions = Family::default();
		let total_subscriptions = Family::default();
		let nats_events = Family::default();
		let commands = Family::default();
		let client_closes = Family::default();

		Self {
			connection_duration_seconds,
			current_connections,
			unique_subscriptions,
			total_subscriptions,
			nats_events,
			commands,
			client_closes,
		}
	}
}

impl MetricsProvider for EventApiMetrics {
	fn register(&mut self, registry: &mut Registry) {
		registry.register(
			"eventapi_connection_duration_seconds",
			"The number of seconds used on connections",
			self.connection_duration_seconds.clone(),
		);

		registry.register(
			"eventapi_current_connections",
			"The current number of connections",
			self.current_connections.clone(),
		);

		registry.register(
			"eventapi_unique_subscriptions",
			"The number of unique subscriptions",
			self.unique_subscriptions.clone(),
		);

		registry.register(
			"eventapi_total_subscriptions",
			"The number of total subscriptions",
			self.total_subscriptions.clone(),
		);

		registry.register("eventapi_nats_events", "The number of NATs events", self.nats_events.clone());

		registry.register("eventapi_commands", "The number of commands issued", self.commands.clone());

		registry.register(
			"eventapi_client_closes",
			"The number of client closes",
			self.client_closes.clone(),
		);
	}
}

impl EventApiMetrics {
	/// Observe a client close code.
	pub fn observe_client_close_event_stream(&self, code: &'static str) {
		self.client_closes.get_or_create(&ClientClose::event_stream(code)).inc();
	}

	/// Observe a client close code.
	pub fn observe_client_close_websocket(&self, code: &'static str) {
		self.client_closes.get_or_create(&ClientClose::websocket(code)).inc();
	}

	/// Observe a nats event miss.
	pub fn observe_nats_event_miss(&self) {
		self.nats_events.get_or_create(&NatsEvent::MISS).inc();
	}

	/// Observe a nats event hit.
	pub fn observe_nats_event_hit(&self) {
		self.nats_events.get_or_create(&NatsEvent::HIT).inc();
	}

	/// Observe how long a connection was open.
	pub fn observe_connection_duration_seconds_event_stream(&self, duration: f64) {
		self.connection_duration_seconds
			.get_or_create(&ConnectionDuration::EVENT_STREAM)
			.observe(duration);
	}

	/// Observe how long a connection was open.
	pub fn observe_connection_duration_seconds_websocket(&self, duration: f64) {
		self.connection_duration_seconds
			.get_or_create(&ConnectionDuration::WEBSOCKET)
			.observe(duration);
	}

	/// Set the number of unique subscriptions.
	pub fn set_unique_subscriptions(&self, len: usize, cap: usize) {
		self.unique_subscriptions
			.get_or_create(&UniqueSubscriptions::LEN)
			.set(len as i64);
		self.unique_subscriptions
			.get_or_create(&UniqueSubscriptions::CAP)
			.set(cap as i64);
	}

	/// Observe a command from the server.
	pub fn observe_server_command(&self, code: Opcode) {
		self.commands.get_or_create(&Command::server(code)).inc();
	}

	/// Observe a command from the client.
	pub fn observe_client_command(&self, code: Opcode) {
		self.commands.get_or_create(&Command::client(code)).inc();
	}

	/// Increment the total subscriptions.
	pub fn incr_total_subscriptions(&self) {
		self.total_subscriptions.get_or_create(&TotalSubscription::V3).inc();
	}

	/// Decrement the total subscriptions.
	pub fn decr_total_subscriptions(&self) {
		self.total_subscriptions.get_or_create(&TotalSubscription::V3).dec();
	}

	/// Increment the total subscriptions.
	pub fn incr_current_event_streams(&self) {
		self.current_connections.get_or_create(&CurrentConnection::EVENT_STREAM).inc();
	}

	/// Decrement the total subscriptions.
	pub fn decr_current_event_streams(&self) {
		self.current_connections.get_or_create(&CurrentConnection::EVENT_STREAM).dec();
	}

	/// Increment the total subscriptions.
	pub fn incr_current_websocket_connections(&self) {
		self.current_connections.get_or_create(&CurrentConnection::WEBSOCKET).inc();
	}

	/// Decrement the total subscriptions.
	pub fn decr_current_websocket_connections(&self) {
		self.current_connections.get_or_create(&CurrentConnection::WEBSOCKET).dec();
	}
}
