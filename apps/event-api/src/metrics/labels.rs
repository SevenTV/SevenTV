use prometheus_client::encoding::EncodeLabelSet;
use shared::event_api::types::Opcode;

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, EncodeLabelSet)]
/// CloseCode labels.
pub struct ClientClose {
	code: &'static str,
	kind: &'static str,
	endpoint: &'static str,
}

impl ClientClose {
	/// Create a new set of labels, the endpoint is always "v3" since we only
	/// have one endpoint for the SSE stream.
	pub const fn event_stream(code: &'static str) -> Self {
		Self {
			code,
			endpoint: "v3",
			kind: "event_stream",
		}
	}

	/// Create a new set of labels, the endpoint is always "v3" since we only
	/// have one endpoint for the Websocket stream.
	pub const fn websocket(code: &'static str) -> Self {
		Self {
			code,
			endpoint: "v3",
			kind: "websocket",
		}
	}
}

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, EncodeLabelSet)]
/// NatsEvent labels.
pub struct NatsEvent {
	kind: &'static str,
}

impl NatsEvent {
	/// Hit event.
	pub const HIT: Self = Self { kind: "hit" };
	/// Miss event.
	pub const MISS: Self = Self { kind: "miss" };
}

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, EncodeLabelSet)]
/// UniqueSubscriptions labels.
pub struct UniqueSubscriptions {
	kind: &'static str,
}

impl UniqueSubscriptions {
	/// Capacity of the unique subscriptions.
	pub const CAP: Self = Self { kind: "cap" };
	/// Length of the unique subscriptions.
	pub const LEN: Self = Self { kind: "len" };
}

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, EncodeLabelSet)]
/// Command labels.
pub struct Command {
	kind: &'static str,
	command: &'static str,
	endpoint: &'static str,
}

impl Command {
	/// Create a new set of labels for a client command, the endpoint is always
	/// "v3" since we only have one endpoint.
	pub const fn client(code: Opcode) -> Self {
		Self {
			kind: "client",
			endpoint: "v3",
			command: code.as_str(),
		}
	}

	/// Create a new set of labels for a server command, the endpoint is always
	/// "v3" since we only have one endpoint.
	pub const fn server(code: Opcode) -> Self {
		Self {
			kind: "server",
			endpoint: "v3",
			command: code.as_str(),
		}
	}
}

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, EncodeLabelSet)]
/// CurrentConnection labels.
pub struct CurrentConnection {
	kind: &'static str,
	endpoint: &'static str,
}

impl CurrentConnection {
	/// Event stream connection.
	pub const EVENT_STREAM: Self = Self {
		kind: "event_stream",
		endpoint: "v3",
	};
	/// Websocket connection.
	pub const WEBSOCKET: Self = Self {
		kind: "websocket",
		endpoint: "v3",
	};
}

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, EncodeLabelSet)]
/// ConnectionDuration labels.
pub struct ConnectionDuration {
	endpoint: &'static str,
	kind: &'static str,
}

impl ConnectionDuration {
	/// Event stream connection.
	pub const EVENT_STREAM: Self = Self {
		endpoint: "v3",
		kind: "event_stream",
	};
	pub const WEBSOCKET: Self = Self {
		endpoint: "v3",
		kind: "websocket",
	};
}

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, EncodeLabelSet)]
/// TotalSubscription labels.
pub struct TotalSubscription {
	endpoint: &'static str,
}

impl TotalSubscription {
	/// V3 endpoint.
	pub const V3: Self = Self { endpoint: "v3" };
}
