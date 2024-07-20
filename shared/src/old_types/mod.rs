//! All types in this module are used by at least two of the following
//! applications:
//! - api
//! - data-brittler
//! - event-api
//!
//! All other old types are defined in the respective application crates.

use bitmask_enum::bitmask;
use cosmetic::{
	CosmeticAvatarModel, CosmeticBadgeModel, CosmeticKind, CosmeticModelAvatar, CosmeticModelBadge, CosmeticModelPaint,
	CosmeticPaintCanvasRepeat, CosmeticPaintFunction, CosmeticPaintGradient, CosmeticPaintGradientStop, CosmeticPaintModel,
	CosmeticPaintShadow, CosmeticPaintShape, CosmeticPaintStroke, CosmeticPaintText, CosmeticPaintTextTransform,
};
use image::{ImageFile, ImageFormat, ImageHost};

use crate::database::badge::BadgeId;
use crate::database::emote::EmoteFlags;
use crate::database::emote_set::{EmoteSet, EmoteSetEmoteFlag, EmoteSetId, EmoteSetKind};
use crate::database::paint::PaintId;
use crate::database::role::permissions::{PermissionsExt, UserPermission};
use crate::database::role::RoleId;
use crate::database::user::connection::{Platform, UserConnection};
use crate::database::user::profile_picture::UserProfilePicture;
use crate::database::user::{FullUser, UserId};

pub mod cosmetic;
pub mod image;
pub mod object_id;
pub mod role_permission;

#[derive(utoipa::OpenApi)]
#[openapi(components(schemas(
	// Emotes
	EmoteFlagsModel,
	// Emote Sets
	EmoteSetFlagModel,
	ActiveEmoteFlagModel,
	// Cosmetic
	CosmeticPaintModel,
	CosmeticModelPaint,
	CosmeticKind,
	CosmeticPaintGradient,
	CosmeticPaintFunction,
	CosmeticPaintGradientStop,
	CosmeticPaintCanvasRepeat,
	CosmeticPaintShadow,
	CosmeticPaintText,
	CosmeticPaintStroke,
	CosmeticPaintTextTransform,
	CosmeticPaintShape,
	CosmeticBadgeModel,
	CosmeticModelBadge,
	CosmeticAvatarModel,
	CosmeticModelAvatar,
	// Image
	ImageHost,
	ImageFile,
	ImageFormat,
	// UserConnection
	UserConnectionPartialModel,
	UserConnectionPlatformModel,
	// User
	UserPartialModel,
	UserStyle,
	UserTypeModel,
)))]
pub struct Docs;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
	value == &T::default()
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user.model.go#L30
pub struct UserPartialModel {
	pub id: UserId,
	#[serde(skip_serializing_if = "is_default")]
	pub user_type: UserTypeModel,
	pub username: String,
	pub display_name: String,
	#[serde(skip_serializing_if = "String::is_empty")]
	pub avatar_url: String,
	pub style: UserStyle,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub role_ids: Vec<RoleId>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub connections: Vec<UserConnectionPartialModel>,
}

impl UserPartialModel {
	pub fn from_db(
		user: FullUser,
		paint: Option<CosmeticPaintModel>,
		badge: Option<CosmeticBadgeModel>,
		profile_picture: Option<UserProfilePicture>,
		cdn_base_url: &str,
	) -> Self {
		let main_connection = user.connections.first();

		let paint_id = user.style.active_paint_id.and_then(|id| {
			if user.has(UserPermission::UsePaint) && user.computed.entitlements.paints.contains(&id) {
				Some(id)
			} else {
				None
			}
		});

		let badge_id = user.style.active_badge_id.and_then(|id| {
			if user.has(UserPermission::UseBadge) && user.computed.entitlements.badges.contains(&id) {
				Some(id)
			} else {
				None
			}
		});

		let badge = badge.and_then(|badge| if Some(badge.id) == badge_id { Some(badge) } else { None });
		let paint = paint.and_then(|paint| if Some(paint.id) == paint_id { Some(paint) } else { None });

		let avatar_url = if user.has(UserPermission::UseCustomProfilePicture) {
			profile_picture
				.map(|p| {
					p.image_set
						.outputs
						.iter()
						.max_by_key(|i| i.height)
						.map(|i| i.get_url(cdn_base_url))
				})
				.flatten()
		} else {
			None
		}
		.or(main_connection.and_then(|c| c.platform_avatar_url.clone()))
		.unwrap_or_default();

		UserPartialModel {
			id: user.id,
			user_type: UserTypeModel::Regular,
			username: main_connection.map(|c| c.platform_username.clone()).unwrap_or_default(),
			display_name: main_connection.map(|c| c.platform_display_name.clone()).unwrap_or_default(),
			avatar_url,
			style: UserStyle {
				color: user
					.computed
					.highest_role_color
					.map(|color| i32::from_be_bytes(color.to_be_bytes()))
					.unwrap_or(0),
				paint_id,
				paint,
				badge_id,
				badge,
			},
			role_ids: user.computed.entitlements.roles.iter().copied().collect(),
			connections: user
				.connections
				.iter()
				.cloned()
				.map(|connection| {
					UserConnectionPartialModel::from_db(
						connection,
						user.style.active_emote_set_id,
						user.computed.permissions.emote_set_capacity.unwrap_or_default(),
					)
				})
				.collect(),
		}
	}
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user.model.go#L49
pub enum UserTypeModel {
	#[default]
	Regular,
	Bot,
	System,
}

async_graphql::scalar!(UserTypeModel);

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user.model.go#L41
pub struct UserStyle {
	#[serde(skip_serializing_if = "is_default")]
	pub color: i32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub paint_id: Option<PaintId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub paint: Option<CosmeticPaintModel>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub badge_id: Option<BadgeId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub badge: Option<CosmeticBadgeModel>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user-connection.model.go#L33
pub struct UserConnectionPartialModel {
	pub id: String,
	pub platform: UserConnectionPlatformModel,
	pub username: String,
	pub display_name: String,
	pub linked_at: u64,
	pub emote_capacity: i32,
	pub emote_set_id: Option<EmoteSetId>,
}

impl UserConnectionPartialModel {
	pub fn from_db(value: UserConnection, emote_set_id: Option<EmoteSetId>, emote_capacity: i32) -> Self {
		Self {
			id: value.platform_id,
			platform: value.platform.into(),
			username: value.platform_username,
			display_name: value.platform_display_name,
			linked_at: value.linked_at.timestamp_millis() as u64,
			emote_capacity,
			emote_set_id,
		}
	}
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user-connection.model.go#L49
pub enum UserConnectionPlatformModel {
	#[default]
	Twitch,
	Youtube,
	Discord,
	Kick,
}

impl From<UserConnectionPlatformModel> for Platform {
	fn from(value: UserConnectionPlatformModel) -> Self {
		match value {
			UserConnectionPlatformModel::Twitch => Self::Twitch,
			UserConnectionPlatformModel::Youtube => Self::Google,
			UserConnectionPlatformModel::Discord => Self::Discord,
			UserConnectionPlatformModel::Kick => Self::Kick,
		}
	}
}

impl From<Platform> for UserConnectionPlatformModel {
	fn from(value: Platform) -> Self {
		match value {
			Platform::Twitch => Self::Twitch,
			Platform::Discord => Self::Discord,
			Platform::Google => Self::Youtube,
			Platform::Kick => Self::Kick,
		}
	}
}

async_graphql::scalar!(UserConnectionPlatformModel, "ConnectionPlatform");

#[bitmask(u32)]
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

impl From<EmoteFlags> for EmoteFlagsModel {
	fn from(value: EmoteFlags) -> Self {
		let mut flags = Self::none();

		if value.contains(EmoteFlags::Private) {
			flags |= Self::Private;
		}

		if value.contains(EmoteFlags::DefaultZeroWidth) {
			flags |= Self::ZeroWidth;
		}

		if value.contains(EmoteFlags::Nsfw) {
			flags |= Self::Sexual;
		}

		flags
	}
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
		let bits = u32::deserialize(deserializer)?;
		Ok(EmoteFlagsModel::from(bits))
	}
}

async_graphql::scalar!(EmoteFlagsModel);

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

#[bitmask(i32)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L55
pub enum ActiveEmoteFlagModel {
	ZeroWidth = 1 << 0,
	OverrideTwitchGlobal = 1 << 16,
	OverrideTwitchSubscriber = 1 << 17,
	OverrideBetterTTV = 1 << 18,
}

async_graphql::scalar!(ActiveEmoteFlagModel);

impl From<EmoteSetEmoteFlag> for ActiveEmoteFlagModel {
	fn from(value: EmoteSetEmoteFlag) -> Self {
		let mut flags = Self::none();

		if value.contains(EmoteSetEmoteFlag::ZeroWidth) {
			flags |= Self::ZeroWidth;
		}

		if value.contains(EmoteSetEmoteFlag::OverrideConflicts) {
			flags |= Self::OverrideBetterTTV | Self::OverrideTwitchGlobal | Self::OverrideTwitchSubscriber;
		}

		flags
	}
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

#[bitmask(i32)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L32
pub enum EmoteSetFlagModel {
	Immutable = 1 << 0,
	Privileged = 1 << 1,
	Personal = 1 << 2,
	Commercial = 1 << 3,
}

impl EmoteSetFlagModel {
	pub fn from_db(value: &EmoteSet) -> Self {
		let mut flags = Self::none();

		match value.kind {
			EmoteSetKind::Global => flags |= Self::Privileged,
			EmoteSetKind::Personal => flags |= Self::Personal,
			EmoteSetKind::Special => flags |= Self::Commercial | Self::Privileged | Self::Immutable,
			EmoteSetKind::Normal => {}
		}

		flags
	}
}

async_graphql::scalar!(EmoteSetFlagModel);

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

#[bitmask(i64)]
// https://github.com/SevenTV/Common/blob/master/structures/v3/type.ban.go#L29
pub enum BanEffect {
	NoPermissions = 1 << 0,
	NoAuth = 1 << 1,
	NoOwnership = 1 << 2,
	MemoryHole = 1 << 3,
	BlockedIp = 1 << 4,
}

async_graphql::scalar!(BanEffect);

impl Default for BanEffect {
	fn default() -> Self {
		BanEffect::none()
	}
}

impl<'a> serde::Deserialize<'a> for BanEffect {
	fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<BanEffect, D::Error> {
		let bits = i64::deserialize(deserializer)?;
		Ok(BanEffect::from(bits))
	}
}

impl serde::Serialize for BanEffect {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.bits().serialize(serializer)
	}
}
