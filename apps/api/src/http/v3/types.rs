use bitflags::bitflags;
use shared::object_id::ObjectId;
use shared::types::old::{ImageHost, UserModelPartial, UserStyle};

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct User {
	pub id: ObjectId,
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
	pub roles: Vec<ObjectId>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub connections: Vec<UserConnection>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct EmoteSetPartial {
	pub id: ObjectId,
	pub name: String,
	pub flags: EmoteSetFlags,
	pub tags: Vec<String>,
	pub capacity: u32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub owner: Option<UserModelPartial>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
pub struct EmoteSetFlags(u32);

bitflags! {
	impl EmoteSetFlags: u32 {
		const IMMUTABLE = 1 << 0;
		const PRIVILEGED = 1 << 1;
		const PERSONAL = 1 << 2;
		const COMMERCIAL = 1 << 3;
	}
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
pub struct UserEditor {
	pub id: ObjectId,
	pub permissions: i32,
	pub visible: bool,
	pub added_at: u64,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
pub struct UserConnection {
	pub id: ObjectId,
	pub platform: String,
	pub username: String,
	pub display_name: String,
	pub linked_at: u64,
	pub emote_capacity: i32,
	pub emote_set_id: Option<ObjectId>,
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
	pub id: ObjectId,
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

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
pub struct EmoteFlags(u32);

bitflags! {
	impl EmoteFlags: u32 {
		const PRIVATE = 1 << 0;
		const AUTHENTIC = 1 << 1;
		const ZERO_WIDTH = 1 << 8;

		const CONTENT_SEXUAL = 1 << 16;
		const CONTENT_EPILEPSY = 1 << 17;
		const CONTENT_EDGY = 1 << 18;
		const CONTENT_TWITCH_DISALLOWED = 1 << 24;
	}
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct EmoteLifecycle(i32);

impl EmoteLifecycle {
	pub const DELETED: Self = Self(-1);
	pub const DISABLED: Self = Self(2);
	pub const FAILED: Self = Self(-2);
	pub const LIVE: Self = Self(3);
	pub const PENDING: Self = Self(0);
	pub const PROCESSING: Self = Self(1);
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct EmoteVersion {
	pub id: ObjectId,
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
