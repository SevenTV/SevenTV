use bitflags::bitflags;
use shared::object_id::ObjectId;
use shared::types::{ImageHost, UserModelPartial};

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
