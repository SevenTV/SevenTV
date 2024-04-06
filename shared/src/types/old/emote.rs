use bitmask_enum::bitmask;
use ulid::Ulid;

use super::{ImageHost, UserPartialModel};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote.model.go#L12
pub struct EmoteModel {
	pub id: Ulid,
	pub name: String,
	pub flags: EmoteFlagsModel,
	pub tags: Vec<String>,
	pub lifecycle: EmoteLifecycleModel,
	pub state: Vec<EmoteVersionState>,
	pub listed: bool,
	pub animated: bool,
	pub owner: Option<UserPartialModel>,
	#[serde(skip)]
	pub owner_id: Ulid,
	pub host: ImageHost,
	pub versions: Vec<EmoteVersionModel>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote.model.go#L27
pub struct EmotePartialModel {
	pub id: Ulid,
	pub name: String,
	pub flags: EmoteFlagsModel,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub tags: Vec<String>,
	pub lifecycle: EmoteLifecycleModel,
	pub state: Vec<EmoteVersionState>,
	pub listed: bool,
	pub animated: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub owner: Option<UserPartialModel>,
	pub host: ImageHost,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote.model.go#L40
pub struct EmoteVersionModel {
	pub id: Ulid,
	pub name: String,
	pub description: String,
	pub lifecycle: EmoteLifecycleModel,
	pub state: Vec<EmoteVersionState>,
	pub listed: bool,
	pub animated: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub host: Option<ImageHost>,
	#[serde(rename = "createdAt")]
	pub created_at: i64,
}

#[derive(Debug, Default, Clone, Copy, utoipa::ToSchema, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote.model.go#L52
/// EmoteLifecycleModel represents the lifecycle of an emote.
/// - `Deleted` (-1): The emote has been deleted.
/// - `Pending` (0): The emote is pending approval.
/// - `Processing` (1): The emote is being processed.
/// - `Disabled` (2): The emote has been disabled.
/// - `Live` (3): The emote is live.
/// - `Failed` (-2): The emote has failed processing.
pub enum EmoteLifecycleModel {
	Deleted = -1,
	#[default]
	Pending = 0,
	Processing = 1,
	Disabled = 2,
	Live = 3,
	Failed = -2,
}

#[bitmask(i32)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote.model.go#L63
pub enum EmoteFlagsModel {
	Private = 1 << 0,
	Authentic = 1 << 1,
	ZeroWidth = 1 << 8,
	Sexual = 1 << 16,
	Epilepsy = 1 << 17,
	Edgy = 1 << 18,
	TwitchDisallowed = 1 << 24,
}

impl Default for EmoteFlagsModel {
	fn default() -> Self {
		EmoteFlagsModel::none()
	}
}

impl serde::Serialize for EmoteFlagsModel {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		self.bits().serialize(serializer)
	}
}

impl<'a> serde::Deserialize<'a> for EmoteFlagsModel {
	fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<EmoteFlagsModel, D::Error> {
		let bits = i32::deserialize(deserializer)?;
		Ok(EmoteFlagsModel::from(bits))
	}
}

impl<'a> utoipa::ToSchema<'a> for EmoteFlagsModel {
	fn schema() -> (&'a str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
		(
			"EmoteFlagsModel",
			utoipa::openapi::ObjectBuilder::new()
				.schema_type(utoipa::openapi::schema::SchemaType::Integer)
				.description(Some(
					"
These are the flags that can be set on an emote. They are represented as a bitmask.

- `Private` (1): The emote is private and can only be used by the owner.

- `Authentic` (2): The emote is authentic and is not a copy of another emote.

- `ZeroWidth` (256): The emote is a zero-width emote.

- `Sexual` (65536): The emote is sexual in nature.

- `Epilepsy` (131072): The emote may cause epilepsy.

- `Edgy` (262144): The emote is edgy.

- `TwitchDisallowed` (16777216): The emote is disallowed on Twitch.
",
				))
				.format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
					utoipa::openapi::KnownFormat::Int32,
				)))
				.into(),
		)
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote.model.go#L78
pub enum EmoteVersionState {
	#[default]
	Listed,
	AllowPersonal,
	NoPersonal,
}
