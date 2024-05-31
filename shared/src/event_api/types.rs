// use hyper_tungstenite::tungstenite::protocol::frame::coding::CloseCode as
// WsCloseCode;

// See the comment on the `payload.rs` file for a description of what this file
// is.
use super::payload::{Subscribe, Unsubscribe};
use crate::{database::Id, old_types::UserPartialModel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Opcode {
	Dispatch = 0,
	Hello = 1,
	Heartbeat = 2,
	Reconnect = 4,
	Ack = 5,
	Error = 6,
	EndOfStream = 7,
	Identify = 33,
	Resume = 34,
	Subscribe = 35,
	Unsubscribe = 36,
	Signal = 37,
	Bridge = 38,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CloseCode {
	ServerError = 4000,
	UnknownOperation = 4001,
	InvalidPayload = 4002,
	AuthFailure = 4003,
	AlreadyIdentified = 4004,
	RateLimit = 4005,
	Restart = 4006,
	Maintenance = 4007,
	Timeout = 4008,
	AlreadySubscribed = 4009,
	NotSubscribed = 4010,
	InsufficientPrivilege = 4011,
	Reconnect = 4012,
}

impl CloseCode {
	pub const fn from_u16(value: u16) -> Option<Self> {
		match value {
			4000 => Some(Self::ServerError),
			4001 => Some(Self::UnknownOperation),
			4002 => Some(Self::InvalidPayload),
			4003 => Some(Self::AuthFailure),
			4004 => Some(Self::AlreadyIdentified),
			4005 => Some(Self::RateLimit),
			4006 => Some(Self::Restart),
			4007 => Some(Self::Maintenance),
			4008 => Some(Self::Timeout),
			4009 => Some(Self::AlreadySubscribed),
			4010 => Some(Self::NotSubscribed),
			4011 => Some(Self::InsufficientPrivilege),
			4012 => Some(Self::Reconnect),
			_ => None,
		}
	}

	pub const fn as_u16(self) -> u16 {
		self as u16
	}

	pub const fn as_code_str(self) -> &'static str {
		match self {
			Self::ServerError => "SERVER_ERROR",
			Self::UnknownOperation => "UNKNOWN_OPERATION",
			Self::InvalidPayload => "INVALID_PAYLOAD",
			Self::AuthFailure => "AUTH_FAILURE",
			Self::AlreadyIdentified => "ALREADY_IDENTIFIED",
			Self::RateLimit => "RATE_LIMIT",
			Self::Restart => "RESTART",
			Self::Maintenance => "MAINTENANCE",
			Self::Timeout => "TIMEOUT",
			Self::AlreadySubscribed => "ALREADY_SUBSCRIBED",
			Self::NotSubscribed => "NOT_SUBSCRIBED",
			Self::InsufficientPrivilege => "INSUFFICIENT_PRIVILEGE",
			Self::Reconnect => "RECONNECT",
		}
	}

	pub const fn as_str(self) -> &'static str {
		match self {
			Self::ServerError => "Internal Server Error",
			Self::UnknownOperation => "Unknown Operation",
			Self::InvalidPayload => "Invalid Payload",
			Self::AuthFailure => "Authentication Failure",
			Self::AlreadyIdentified => "Already identified",
			Self::RateLimit => "Rate limit reached",
			Self::Restart => "Server is restarting",
			Self::Maintenance => "Maintenance Mode",
			Self::Timeout => "Timeout",
			Self::AlreadySubscribed => "Already Subscribed",
			Self::NotSubscribed => "Not Subscribed",
			Self::InsufficientPrivilege => "Insufficient Privilege",
			Self::Reconnect => "Reconnect",
		}
	}

	// pub const fn into_websocket(self) -> WsCloseCode {
	// 	WsCloseCode::Library(self.as_u16())
	// }
}

impl std::fmt::Display for CloseCode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}

impl Opcode {
	pub const fn from_u16(value: u16) -> Option<Self> {
		match value {
			0 => Some(Self::Dispatch),
			1 => Some(Self::Hello),
			2 => Some(Self::Heartbeat),
			4 => Some(Self::Reconnect),
			5 => Some(Self::Ack),
			6 => Some(Self::Error),
			7 => Some(Self::EndOfStream),
			33 => Some(Self::Identify),
			34 => Some(Self::Resume),
			35 => Some(Self::Subscribe),
			36 => Some(Self::Unsubscribe),
			37 => Some(Self::Signal),
			38 => Some(Self::Bridge),
			_ => None,
		}
	}

	pub const fn as_u16(self) -> u16 {
		self as u16
	}

	pub const fn as_str(self) -> &'static str {
		match self {
			Self::Dispatch => "DISPATCH",
			Self::Hello => "HELLO",
			Self::Heartbeat => "HEARTBEAT",
			Self::Reconnect => "RECONNECT",
			Self::Ack => "ACK",
			Self::Error => "ERROR",
			Self::EndOfStream => "END_OF_STREAM",
			Self::Identify => "IDENTIFY",
			Self::Resume => "RESUME",
			Self::Subscribe => "SUBSCRIBE",
			Self::Unsubscribe => "UNSUBSCRIBE",
			Self::Signal => "SIGNAL",
			Self::Bridge => "BRIDGE",
		}
	}
}

impl std::fmt::Display for Opcode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}

impl serde::Serialize for Opcode {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_u16(self.as_u16())
	}
}

impl serde::Serialize for CloseCode {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_u16(self.as_u16())
	}
}

impl<'a> serde::Deserialize<'a> for Opcode {
	fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
		let s = u16::deserialize(deserializer)?;
		Self::from_u16(s).ok_or_else(|| serde::de::Error::custom("invalid opcode"))
	}
}

impl<'a> serde::Deserialize<'a> for CloseCode {
	fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
		let s = u16::deserialize(deserializer)?;
		Self::from_u16(s).ok_or_else(|| serde::de::Error::custom("invalid close code"))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EventType {
	AnySystem,
	SystemAnnouncement,

	AnyEmote,
	CreateEmote,
	UpdateEmote,
	DeleteEmote,

	AnyEmoteSet,
	CreateEmoteSet,
	UpdateEmoteSet,
	DeleteEmoteSet,

	AnyUser,
	CreateUser,
	UpdateUser,
	DeleteUser,

	AnyEntitlement,
	CreateEntitlement,
	UpdateEntitlement,
	DeleteEntitlement,

	AnyCosmetic,
	CreateCosmetic,
	UpdateCosmetic,
	DeleteCosmetic,

	Whisper,
}

impl EventType {
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::AnySystem => "system.*",
			Self::SystemAnnouncement => "system.announcement",
			Self::AnyEmote => "emote.*",
			Self::CreateEmote => "emote.create",
			Self::UpdateEmote => "emote.update",
			Self::DeleteEmote => "emote.delete",
			Self::AnyEmoteSet => "emote_set.*",
			Self::CreateEmoteSet => "emote_set.create",
			Self::UpdateEmoteSet => "emote_set.update",
			Self::DeleteEmoteSet => "emote_set.delete",
			Self::AnyUser => "user.*",
			Self::CreateUser => "user.create",
			Self::UpdateUser => "user.update",
			Self::DeleteUser => "user.delete",
			Self::AnyEntitlement => "entitlement.*",
			Self::CreateEntitlement => "entitlement.create",
			Self::UpdateEntitlement => "entitlement.update",
			Self::DeleteEntitlement => "entitlement.delete",
			Self::AnyCosmetic => "cosmetic.*",
			Self::CreateCosmetic => "cosmetic.create",
			Self::UpdateCosmetic => "cosmetic.update",
			Self::DeleteCosmetic => "cosmetic.delete",
			Self::Whisper => "whisper.self",
		}
	}
}

impl std::fmt::Display for EventType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}

impl std::str::FromStr for EventType {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"system.*" => Ok(Self::AnySystem),
			"system.announcement" => Ok(Self::SystemAnnouncement),
			"emote.*" => Ok(Self::AnyEmote),
			"emote.create" => Ok(Self::CreateEmote),
			"emote.update" => Ok(Self::UpdateEmote),
			"emote.delete" => Ok(Self::DeleteEmote),
			"emote_set.*" => Ok(Self::AnyEmoteSet),
			"emote_set.create" => Ok(Self::CreateEmoteSet),
			"emote_set.update" => Ok(Self::UpdateEmoteSet),
			"emote_set.delete" => Ok(Self::DeleteEmoteSet),
			"user.*" => Ok(Self::AnyUser),
			"user.create" => Ok(Self::CreateUser),
			"user.update" => Ok(Self::UpdateUser),
			"user.delete" => Ok(Self::DeleteUser),
			"entitlement.*" => Ok(Self::AnyEntitlement),
			"entitlement.create" => Ok(Self::CreateEntitlement),
			"entitlement.update" => Ok(Self::UpdateEntitlement),
			"entitlement.delete" => Ok(Self::DeleteEntitlement),
			"cosmetic.*" => Ok(Self::AnyCosmetic),
			"cosmetic.create" => Ok(Self::CreateCosmetic),
			"cosmetic.update" => Ok(Self::UpdateCosmetic),
			"cosmetic.delete" => Ok(Self::DeleteCosmetic),
			"whisper.self" => Ok(Self::Whisper),
			_ => Err(()),
		}
	}
}

impl serde::Serialize for EventType {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(self.as_str())
	}
}

impl<'a> serde::Deserialize<'a> for EventType {
	fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
		let s = String::deserialize(deserializer)?;
		s.parse().map_err(|_| serde::de::Error::custom("invalid event type"))
	}
}

fn is_false(v: &bool) -> bool {
	!*v
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeMap {
	#[serde(default)]
	pub id: Id,
	pub kind: ObjectKind,
	#[serde(skip_serializing_if = "is_false")]
	#[serde(default)]
	pub contextual: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub actor: Option<UserPartialModel>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	#[serde(default)]
	pub added: Vec<ChangeField>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	#[serde(default)]
	pub updated: Vec<ChangeField>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	#[serde(default)]
	pub removed: Vec<ChangeField>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	#[serde(default)]
	pub pushed: Vec<ChangeField>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	#[serde(default)]
	pub pulled: Vec<ChangeField>,
	#[serde(skip_serializing_if = "serde_json::Value::is_null")]
	#[serde(default)]
	pub object: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeField {
	pub key: String,
	#[serde(default)]
	pub index: Option<usize>,
	#[serde(skip_serializing_if = "is_false")]
	#[serde(default)]
	pub nested: bool,
	#[serde(rename = "type")]
	pub ty: ChangeFieldType,
	#[serde(skip_serializing_if = "serde_json::Value::is_null")]
	#[serde(default)]
	pub old_value: serde_json::Value,
	#[serde(default)]
	pub value: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ObjectKind {
	User = 1,
	Emote = 2,
	EmoteSet = 3,
	Role = 4,
	Entitlement = 5,
	Ban = 6,
	Message = 7,
	Report = 8,
	Presence = 9,
	Cosmetic = 10,
}

impl ObjectKind {
	pub const fn from_u16(value: u16) -> Option<Self> {
		match value {
			1 => Some(Self::User),
			2 => Some(Self::Emote),
			3 => Some(Self::EmoteSet),
			4 => Some(Self::Role),
			5 => Some(Self::Entitlement),
			6 => Some(Self::Ban),
			7 => Some(Self::Message),
			8 => Some(Self::Report),
			9 => Some(Self::Presence),
			10 => Some(Self::Cosmetic),
			_ => None,
		}
	}

	pub const fn as_u16(self) -> u16 {
		self as u16
	}

	pub const fn as_str(self) -> &'static str {
		match self {
			Self::User => "USER",
			Self::Emote => "EMOTE",
			Self::EmoteSet => "EMOTE_SET",
			Self::Role => "ROLE",
			Self::Entitlement => "ENTITLEMENT",
			Self::Ban => "BAN",
			Self::Message => "MESSAGE",
			Self::Report => "REPORT",
			Self::Presence => "PRESENCE",
			Self::Cosmetic => "COSMETIC",
		}
	}
}

impl std::fmt::Display for ObjectKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}

impl serde::Serialize for ObjectKind {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_u16(self.as_u16())
	}
}

impl<'a> serde::Deserialize<'a> for ObjectKind {
	fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
		let s = u16::deserialize(deserializer)?;
		Self::from_u16(s).ok_or_else(|| serde::de::Error::custom("invalid object kind"))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeFieldType {
	String,
	Number,
	Bool,
	Object,
	Empty,
}

impl ChangeFieldType {
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::String => "string",
			Self::Number => "number",
			Self::Bool => "bool",
			Self::Object => "object",
			Self::Empty => "",
		}
	}
}

impl std::fmt::Display for ChangeFieldType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}

impl serde::Serialize for ChangeFieldType {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(self.as_str())
	}
}

impl<'a> serde::Deserialize<'a> for ChangeFieldType {
	fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
		let s = String::deserialize(deserializer)?;
		match s.as_str() {
			"string" => Ok(Self::String),
			"number" => Ok(Self::Number),
			"bool" => Ok(Self::Bool),
			"object" => Ok(Self::Object),
			"" => Ok(Self::Empty),
			_ => Err(serde::de::Error::custom("invalid change field type")),
		}
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct SessionEffect {
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub add_subscriptions: Vec<Subscribe>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub remove_subscriptions: Vec<Unsubscribe>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub remove_hashes: Vec<u32>,
}
