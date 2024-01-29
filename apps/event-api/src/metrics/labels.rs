use std::sync::Arc;

use prometheus_client::encoding::{EncodeLabelKey, EncodeLabelSet, EncodeLabelValue};

use crate::message::types::Opcode;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// Labels<T> is a wrapper around a set of labels.
/// This is advantageous because it allows us to have a base set of labels that
/// are always present, and then extend them with additional labels. Without
/// copying the base labels.
pub struct Labels<T> {
	base_labels: Arc<[(String, String)]>,
	ext: T,
}

impl Labels<()> {
	/// Create a new set of labels.
	pub fn new(base_labels: Vec<(String, String)>) -> Self {
		Self {
			base_labels: base_labels.into(),
			ext: (),
		}
	}
}

impl<T> Labels<T> {
	/// Extend the labels with additional labels.
	pub fn extend<Y>(&self, ext: Y) -> Labels<Y> {
		Labels {
			base_labels: self.base_labels.clone(),
			ext,
		}
	}
}

/// A custom implementation of EncodeLabelSet for Labels<T>.
impl<T: EncodeLabelSet> EncodeLabelSet for Labels<T> {
	fn encode(&self, mut encoder: prometheus_client::encoding::LabelSetEncoder) -> Result<(), std::fmt::Error> {
		for (key, value) in self.base_labels.iter() {
			let mut label_encoder = encoder.encode_label();
			let mut label_key_encoder = label_encoder.encode_label_key()?;
			EncodeLabelKey::encode(key, &mut label_key_encoder)?;
			let mut label_value_encoder = label_key_encoder.encode_label_value()?;
			EncodeLabelValue::encode(value, &mut label_value_encoder)?;
			label_value_encoder.finish()?;
		}

		self.ext.encode(encoder)
	}
}

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, EncodeLabelSet)]
/// CloseCode labels.
pub struct ClientClose {
	code: &'static str,
	endpoint: &'static str,
}

impl ClientClose {
	/// Create a new set of labels, the endpoint is always "v3" since we only
	/// have one endpoint.
	pub const fn new(code: &'static str) -> Self {
		Self { code, endpoint: "v3" }
	}
}

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, EncodeLabelSet)]
/// Memory labels.
pub struct Memory {
	kind: &'static str,
}

impl Memory {
	/// Allocated memory.
	pub const ALLOCATED: Self = Self { kind: "allocated" };
	/// Free memory.
	pub const REMAINING: Self = Self { kind: "remaining" };
	/// Virtual memory.
	pub const RESIDENT: Self = Self { kind: "resident" };
	/// Virtual memory.
	pub const VIRTUAL: Self = Self { kind: "virtual" };
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
}

impl ConnectionDuration {
	/// V3 endpoint.
	pub const V3: Self = Self { endpoint: "v3" };
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
