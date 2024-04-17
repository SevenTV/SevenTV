use bitmask_enum::bitmask;
use bson::oid::ObjectId;

use super::{is_default, EmotePartialModel, UserPartialModel};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L9
pub struct EmoteSetModel {
	pub id: ObjectId,
	pub name: String,
	pub flags: EmoteSetFlagModel,
	pub tags: Vec<String>,
	pub immutable: bool,
	pub privileged: bool,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub emotes: Vec<ActiveEmoteModel>,
	#[serde(skip_serializing_if = "is_default")]
	pub emote_count: i32,
	pub capacity: i32,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub origins: Vec<EmoteSetOrigin>,
	pub owner: Option<UserPartialModel>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L23
pub struct EmoteSetPartialModel {
	pub id: ObjectId,
	pub name: String,
	pub flags: EmoteSetFlagModel,
	pub tags: Vec<String>,
	pub capacity: i32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub owner: Option<UserPartialModel>,
}

#[bitmask(i32)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L32
pub enum EmoteSetFlagModel {
	Immutable = 1 << 0,
	Privileged = 1 << 1,
	Personal = 1 << 2,
	Commercial = 1 << 3,
}

impl Default for EmoteSetFlagModel {
	fn default() -> Self {
		EmoteSetFlagModel::none()
	}
}

impl serde::Serialize for EmoteSetFlagModel {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.bits().serialize(serializer)
	}
}

impl<'a> serde::Deserialize<'a> for EmoteSetFlagModel {
	fn deserialize<D>(deserializer: D) -> Result<EmoteSetFlagModel, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let bits = i32::deserialize(deserializer)?;
		Ok(EmoteSetFlagModel::from(bits))
	}
}

impl<'a> utoipa::ToSchema<'a> for EmoteSetFlagModel {
	fn schema() -> (&'a str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
		(
			"EmoteSetFlagModel",
			utoipa::openapi::ObjectBuilder::new()
				.schema_type(utoipa::openapi::schema::SchemaType::Integer)
				.description(Some(
					"These flags are used to determine the behavior of the emote set.

- `Immutable` (1): The emote set is immutable and cannot be modified.

- `Privileged` (2): The emote set is privileged and can be used by privileged users.

- `Personal` (4): The emote set is personal and can only be used by the owner.

- `Commercial` (8): The emote set is commercial and can be used for commercial purposes.",
				))
				.format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
					utoipa::openapi::KnownFormat::Int32,
				)))
				.into(),
		)
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L45
pub struct ActiveEmoteModel {
	pub id: ObjectId,
	pub name: String,
	pub flags: ActiveEmoteFlagModel,
	pub timestamp: i64,
	pub actor_id: Option<ObjectId>,
	pub data: Option<EmotePartialModel>,
	pub origin_id: Option<ObjectId>,
}

#[bitmask(i32)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L55
pub enum ActiveEmoteFlagModel {
	ZeroWidth = 1 << 0,
	OverrideTwitchGlobal = 1 << 16,
	OverrideTwitchSubscriber = 1 << 17,
	OverrideBetterTTV = 1 << 18,
}

impl Default for ActiveEmoteFlagModel {
	fn default() -> Self {
		ActiveEmoteFlagModel::none()
	}
}

impl serde::Serialize for ActiveEmoteFlagModel {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.bits().serialize(serializer)
	}
}

impl<'a> serde::Deserialize<'a> for ActiveEmoteFlagModel {
	fn deserialize<D>(deserializer: D) -> Result<ActiveEmoteFlagModel, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let bits = i32::deserialize(deserializer)?;
		Ok(ActiveEmoteFlagModel::from(bits))
	}
}

impl<'a> utoipa::ToSchema<'a> for ActiveEmoteFlagModel {
	fn schema() -> (&'a str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
		(
			"ActiveEmoteFlagModel",
			utoipa::openapi::ObjectBuilder::new()
				.schema_type(utoipa::openapi::schema::SchemaType::Integer)
				.description(Some(
					"These flags are used to determine the behavior of the active emote.

- `ZeroWidth` (1): The emote is a zero-width emote.

- `OverrideTwitchGlobal` (65536): The emote overrides the global Twitch emote.

- `OverrideTwitchSubscriber` (131072): The emote overrides the Twitch subscriber emote.

- `OverrideBetterTTV` (262144): The emote overrides the BetterTTV emote.",
				))
				.format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
					utoipa::openapi::KnownFormat::Int32,
				)))
				.into(),
		)
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L64
pub struct EmoteSetOrigin {
	pub id: ObjectId,
	pub weight: i32,
	pub slices: Vec<u32>,
}
