use std::collections::HashMap;

// There is a lot of code in this file. I will try to explain it as best as I can.
// These are the structs that were used in the previous implementation of the event-api, for the payloads.
// When events are dispatched on NATs or received from the websocket, they SHOULD be in the form of these structs.
// All of these structs have #[serde(default)] and #[serde(deny_unknown_fields)], this is because if we receive a
// payload, with a value we don't recognize rather than ignoring it (which was previously the case), we will now error.
// The reason this is desirable is because if we made a mistake in one of the payloads here we would like to know about
// it rather than silently ignoring it, and potentially causing issues.
use super::types::{self, ChangeMap, CloseCode, EventType, SessionEffect};
use super::MessagePayload;
use crate::database::{Id, UserId};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Hello {
	pub heartbeat_interval: u32,
	pub session_id: Id,
	pub subscription_limit: i32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub actor: Option<UserId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub instance: Option<HelloInstanceInfo>,
}

impl MessagePayload for Hello {
	fn opcode(&self) -> types::Opcode {
		types::Opcode::Hello
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct HelloInstanceInfo {
	pub name: String,
	pub population: i32,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Ack {
	pub command: String,
	pub data: serde_json::Value,
}

impl MessagePayload for Ack {
	fn opcode(&self) -> types::Opcode {
		types::Opcode::Ack
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Heartbeat {
	pub count: u64,
}

impl MessagePayload for Heartbeat {
	fn opcode(&self) -> types::Opcode {
		types::Opcode::Heartbeat
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Reconnect {
	pub reason: String,
}

impl MessagePayload for Reconnect {
	fn opcode(&self) -> types::Opcode {
		types::Opcode::Reconnect
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Resume {
	pub session_id: String,
}

impl MessagePayload for Resume {
	fn opcode(&self) -> types::Opcode {
		types::Opcode::Resume
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subscribe {
	#[serde(rename = "type")]
	pub ty: EventType,
	#[serde(default)]
	pub condition: HashMap<String, String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub ttl: Option<i64>,
}

impl MessagePayload for Subscribe {
	fn opcode(&self) -> types::Opcode {
		types::Opcode::Subscribe
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unsubscribe {
	#[serde(rename = "type")]
	pub ty: EventType,
	#[serde(default)]
	pub condition: HashMap<String, String>,
}

impl MessagePayload for Unsubscribe {
	fn opcode(&self) -> types::Opcode {
		types::Opcode::Unsubscribe
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dispatch {
	#[serde(rename = "type")]
	pub ty: EventType,
	pub body: ChangeMap,
	#[serde(skip_serializing)]
	#[serde(default)]
	pub hash: Option<u32>,
	#[serde(skip_serializing)]
	#[serde(default)]
	pub effect: Option<SessionEffect>,
	#[serde(skip_serializing)]
	#[serde(default)]
	pub matches: Vec<u32>,
	#[serde(skip_serializing)]
	#[serde(default)]
	pub condition: Vec<HashMap<String, String>>,
	#[serde(skip_serializing)]
	#[serde(default)]
	pub whisper: Option<String>,
}

impl MessagePayload for Dispatch {
	fn opcode(&self) -> types::Opcode {
		types::Opcode::Dispatch
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Signal {
	pub sender: SignalUser,
	pub host: SignalUser,
}

impl MessagePayload for Signal {
	fn opcode(&self) -> types::Opcode {
		types::Opcode::Signal
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct SignalUser {
	pub id: UserId,
	pub channel_id: String,
	pub username: String,
	pub display_name: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Error {
	pub message: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub message_locale: Option<String>,
	pub fields: HashMap<String, serde_json::Value>,
}

impl MessagePayload for Error {
	fn opcode(&self) -> types::Opcode {
		types::Opcode::Error
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndOfStream {
	pub code: CloseCode,
	pub message: String,
}

impl MessagePayload for EndOfStream {
	fn opcode(&self) -> types::Opcode {
		types::Opcode::EndOfStream
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Bridge {
	pub command: String,
	pub sid: String,
	pub ip: String,
	pub body: serde_json::Value,
}

impl MessagePayload for Bridge {
	fn opcode(&self) -> types::Opcode {
		types::Opcode::Bridge
	}
}
