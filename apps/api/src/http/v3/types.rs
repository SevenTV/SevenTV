use bitmask_enum::bitmask;
use serde::{Deserialize, Serialize};
use shared::types::old::{ImageHost, UserModelPartial, UserStyle};
use ulid::Ulid;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct User {
	pub id: Ulid,
	#[serde(rename = "type", skip_serializing_if = "String::is_empty")]
	pub ty: String,
	pub username: String,
	pub display_name: String,
	pub created_at: u64,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub avatar_url: Option<String>,
	#[serde(skip_serializing_if = "String::is_empty")]
	pub biography: String,
	pub style: UserStyle,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub emote_sets: Vec<EmoteSetPartial>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub editors: Vec<UserEditor>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub roles: Vec<Ulid>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub connections: Vec<UserConnection>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct EmoteSetPartial {
	pub id: Ulid,
	pub name: String,
	pub flags: EmoteSetFlags,
	pub tags: Vec<String>,
	pub capacity: u32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub owner: Option<UserModelPartial>,
}

#[derive(utoipa::ToSchema)]
#[bitmask(u8)]
pub enum EmoteSetFlags {
	Immutable = 0b0001,
	Privileged = 0b0010,
	Personal = 0b0100,
	Commercial = 0b1000,
}

impl Default for EmoteSetFlags {
	fn default() -> Self {
		Self::none()
	}
}

impl Serialize for EmoteSetFlags {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serde::Serialize::serialize(&self.bits(), serializer)
	}
}

impl<'de> Deserialize<'de> for EmoteSetFlags {
	fn deserialize<'d, D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let bits = u8::deserialize(deserializer)?;
		Ok(Self::from(bits))
	}
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
pub struct UserEditor {
	pub id: Ulid,
	pub permissions: i32,
	pub visible: bool,
	pub added_at: u64,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
pub struct UserConnection {
	pub id: Ulid,
	pub platform: String,
	pub username: String,
	pub display_name: String,
	pub linked_at: u64,
	pub emote_capacity: i32,
	pub emote_set_id: Option<Ulid>,
	pub emote_set: Option<EmoteSetPartial>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub presences: Vec<UserModelPartial>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub user: Option<User>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Emote {
	pub id: Ulid,
	pub name: String,
	pub flags: EmoteFlags,
	pub tags: Vec<String>,
	pub lifecycle: EmoteLifecycle,
	pub state: Vec<EmoteVersionState>,
	pub listed: bool,
	pub animated: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub owner: Option<UserModelPartial>,
	pub host: ImageHost,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub versions: Vec<EmoteVersion>,
}

#[derive(utoipa::ToSchema)]
#[bitmask(u32)]
pub enum EmoteFlags {
	Private = { 1 << 0 },
	Authentic = { 1 << 1 },
	ZeroWidth = { 1 << 8 },

	ContentSexual = { 1 << 16 },
	ContentEpilepsy = { 1 << 17 },
	ContentEdgy = { 1 << 18 },
	ContentTwitchDisallowed = { 1 << 24 },
}

impl Default for EmoteFlags {
	fn default() -> Self {
		Self::none()
	}
}

impl Serialize for EmoteFlags {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serde::Serialize::serialize(&self.bits(), serializer)
	}
}

impl<'de> Deserialize<'de> for EmoteFlags {
	fn deserialize<'d, D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let bits = u32::deserialize(deserializer)?;
		Ok(Self::from(bits))
	}
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[repr(i32)]
pub enum EmoteLifecycle {
	Deleted = -1,
	Disabled = 2,
	Failed = -2,
	Live = 3,
	#[default]
	Pending = 0,
	Processing = 1,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct EmoteVersion {
	pub id: Ulid,
	pub name: String,
	pub description: String,
	pub lifecycle: EmoteLifecycle,
	pub state: Vec<EmoteVersionState>,
	pub listed: bool,
	pub animated: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub host: Option<ImageHost>,
	pub created_at: u64,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmoteVersionState {
	Listed,
	Personal,
	NoPersonal,
}
