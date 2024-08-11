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
use crate::database::emote::{Emote, EmoteFlags, EmoteId};
use crate::database::emote_set::{EmoteSet, EmoteSetEmote, EmoteSetEmoteFlag, EmoteSetId, EmoteSetKind};
use crate::database::paint::PaintId;
use crate::database::role::permissions::{PermissionsExt, UserPermission};
use crate::database::role::RoleId;
use crate::database::user::connection::{Platform, UserConnection};
use crate::database::user::editor::{EditorEmotePermission, EditorEmoteSetPermission, EditorUserPermission, UserEditor, UserEditorPermissions, UserEditorState};
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
			user.active_profile_picture
				.as_ref()
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L64
pub struct EmoteSetOrigin {
	pub id: EmoteSetId,
	pub weight: i32,
	pub slices: Vec<u32>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L9
pub struct EmoteSetModel {
	pub id: EmoteSetId,
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

impl EmoteSetModel {
	pub fn from_db(
		value: EmoteSet,
		emotes: impl IntoIterator<Item = (EmoteSetEmote, Option<EmotePartialModel>)>,
		owner: Option<UserPartialModel>,
	) -> Self {
		let emotes = emotes
			.into_iter()
			.map(|(emote, data)| ActiveEmoteModel::from_db(emote, data))
			.collect::<Vec<_>>();

		Self {
			flags: EmoteSetFlagModel::from_db(&value),
			id: value.id.into(),
			name: value.name,
			tags: value.tags,
			immutable: match value.kind {
				EmoteSetKind::Special => true,
				_ => false,
			},
			privileged: match value.kind {
				EmoteSetKind::Special | EmoteSetKind::Global => true,
				_ => false,
			},
			emote_count: emotes.len() as i32,
			capacity: value.capacity.unwrap_or(i32::MAX) as i32,
			emotes,
			origins: value.origin_config.map_or_else(Vec::new, |config| {
				config
					.origins
					.iter()
					.enumerate()
					.map(|(idx, origin)| EmoteSetOrigin {
						id: origin.id.into(),
						weight: idx as i32,
						slices: Vec::new(),
					})
					.collect()
			}),
			owner,
		}
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L23
pub struct EmoteSetPartialModel {
	pub id: EmoteSetId,
	pub name: String,
	pub flags: EmoteSetFlagModel,
	pub tags: Vec<String>,
	pub capacity: i32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub owner: Option<UserPartialModel>,
}

impl EmoteSetPartialModel {
	pub fn from_db(value: EmoteSet, owner: Option<UserPartialModel>) -> Self {
		EmoteSetPartialModel {
			flags: EmoteSetFlagModel::from_db(&value),
			id: value.id,
			name: value.name,
			capacity: value.capacity.unwrap_or(i32::MAX) as i32,
			tags: value.tags,
			owner,
		}
	}
}

// https://github.com/SevenTV/Common/blob/master/structures/v3/type.user.go#L220
#[bitmask(u32)]
pub enum UserEditorModelPermission {
	ModifyEmotes = 1 << 0,
	UsePrivateEmotes = 1 << 1,
	ManageProfile = 1 << 2,
	ManageOwnedEmotes = 1 << 3,
	ManageEmoteSets = 1 << 4,
	ManageBilling = 1 << 5,
	ManageEditors = 1 << 6,
	ViewMessages = 1 << 7,
}

impl UserEditorModelPermission {
	pub fn from_db(value: &UserEditorPermissions) -> Self {
		let mut perms = Self::none();

		if value.has_emote_set(EditorEmoteSetPermission::Create) || value.has_emote_set(EditorEmoteSetPermission::Manage) {
			perms |= Self::ManageEmoteSets | Self::ModifyEmotes;
		}

		if value.has_emote(EditorEmotePermission::Manage) || value.has_emote(EditorEmotePermission::Create) {
			perms |= Self::ManageOwnedEmotes;
		}

		if value.has_user(EditorUserPermission::ManageBilling) {
			perms |= Self::ManageBilling;
		}
		if value.has_user(EditorUserPermission::ManageProfile) {
			perms |= Self::ManageProfile;
		}
		if value.has_user(EditorUserPermission::ManageEditors) {
			perms |= Self::ManageEditors;
		}

		perms
	}

	pub fn to_db(&self) -> UserEditorPermissions {
		let mut perms = UserEditorPermissions::default();

		if self.contains(Self::ManageEmoteSets) {
			perms.emote_set |= EditorEmoteSetPermission::Create | EditorEmoteSetPermission::Manage;
		}

		if self.contains(Self::ManageOwnedEmotes) {
			perms.emote |= EditorEmotePermission::Create | EditorEmotePermission::Manage;
		}

		if self.contains(Self::ManageBilling) {
			perms.user |= EditorUserPermission::ManageBilling;
		}

		if self.contains(Self::ManageProfile) {
			perms.user |= EditorUserPermission::ManageProfile;
		}

		if self.contains(Self::ManageEditors) {
			perms.user |= EditorUserPermission::ManageEditors;
		}

		perms
	}
}

async_graphql::scalar!(UserEditorModelPermission);

impl Default for UserEditorModelPermission {
	fn default() -> Self {
		UserEditorModelPermission::none()
	}
}

impl serde::Serialize for UserEditorModelPermission {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.bits().serialize(serializer)
	}
}

impl<'a> serde::Deserialize<'a> for UserEditorModelPermission {
	fn deserialize<D>(deserializer: D) -> Result<UserEditorModelPermission, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let bits = u32::deserialize(deserializer)?;
		Ok(UserEditorModelPermission::from(bits))
	}
}

impl<'a> utoipa::ToSchema<'a> for UserEditorModelPermission {
	fn schema() -> (&'a str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
		(
			"UserEditorModelPermission",
			utoipa::openapi::ObjectBuilder::new()
				.schema_type(utoipa::openapi::schema::SchemaType::Integer)
				.description(Some(
					"These flags are used to define what permissions a user editor has.

- `ModifyEmotes` (1): Allows modifying emotes in the user's active emote sets

- `UsePrivateEmotes` (2): Allows using the user's private emotes

- `ManageProfile` (4): Allows managing the user's public profile

- `ManageOwnedEmotes` (8): Allows managing the user's owned emotes

- `ManageEmoteSets` (16): Allows managing the user's owned emote sets

- `ManageBilling` (32): Allows managing billing and payments, such as subscriptions

- `ManageEditors` (64): Allows adding or removing editors for the user

- `ViewMessages` (128): Allows viewing the user's private messages, such as inbox",
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
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user.model.go#L171
pub struct UserEditorModel {
	pub id: UserId,
	pub permissions: UserEditorModelPermission,
	pub visible: bool,
	pub added_at: u64,
}

impl UserEditorModel {
	pub fn from_db(value: UserEditor) -> Option<Self> {
		if value.state != UserEditorState::Accepted {
			return None;
		}

		Some(Self {
			id: value.id.editor_id,
			added_at: value.added_at.timestamp_millis() as u64,
			permissions: UserEditorModelPermission::ModifyEmotes | UserEditorModelPermission::ManageEmoteSets,
			visible: true,
		})
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user.model.go#L15
pub struct UserModel {
	pub id: UserId,
	#[serde(skip_serializing_if = "is_default")]
	pub user_type: UserTypeModel,
	pub username: String,
	pub display_name: String,
	#[serde(skip_serializing_if = "is_default")]
	pub created_at: u64,
	#[serde(skip_serializing_if = "String::is_empty")]
	pub avatar_url: String,
	#[serde(skip_serializing_if = "String::is_empty")]
	pub biography: String,
	pub style: UserStyle,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub emote_sets: Vec<EmoteSetPartialModel>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub editors: Vec<UserEditorModel>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub roles: Vec<RoleId>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub connections: Vec<UserConnectionModel>,
}

impl UserModel {
	pub fn from_db(
		user: FullUser,
		paint: Option<CosmeticPaintModel>,
		badge: Option<CosmeticBadgeModel>,
		emote_sets: Vec<EmoteSetPartialModel>,
		editors: Vec<UserEditorModel>,
		cdn_base_url: &str,
	) -> Self {
		let created_at = user.id.timestamp_ms();
		let active_emote_set_id = user.style.active_emote_set_id;
		let partial = UserPartialModel::from_db(user, paint, badge, cdn_base_url);

		Self {
			id: partial.id,
			user_type: partial.user_type,
			username: partial.username,
			display_name: partial.display_name,
			created_at,
			avatar_url: partial.avatar_url,
			biography: String::new(),
			style: partial.style,
			emote_sets,
			editors,
			roles: partial.role_ids,
			connections: partial
				.connections
				.into_iter()
				.map(|p| UserConnectionModel {
					id: p.id,
					platform: p.platform,
					username: p.username,
					display_name: p.display_name,
					linked_at: p.linked_at,
					emote_capacity: p.emote_capacity,
					emote_set_id: active_emote_set_id,
					emote_set: None,
					user: None,
				})
				.collect(),
		}
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user-connection.model.go#L10
pub struct UserConnectionModel {
	pub id: String,
	pub platform: UserConnectionPlatformModel,
	pub username: String,
	pub display_name: String,
	pub linked_at: u64,
	pub emote_capacity: i32,
	pub emote_set_id: Option<EmoteSetId>,
	pub emote_set: Option<EmoteSetModel>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub user: Option<UserModel>,
}

impl From<UserConnectionPartialModel> for UserConnectionModel {
	fn from(value: UserConnectionPartialModel) -> Self {
		Self {
			id: value.id,
			platform: value.platform,
			username: value.username,
			display_name: value.display_name,
			linked_at: value.linked_at,
			emote_capacity: value.emote_capacity,
			emote_set_id: value.emote_set_id,
			emote_set: None,
			user: None,
		}
	}
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L45
pub struct ActiveEmoteModel {
	pub id: EmoteId,
	pub name: String,
	pub flags: ActiveEmoteFlagModel,
	pub timestamp: i64,
	pub actor_id: Option<UserId>,
	pub data: Option<EmotePartialModel>,
	pub origin_id: Option<EmoteSetId>,
}

impl ActiveEmoteModel {
	pub fn from_db(value: EmoteSetEmote, data: Option<EmotePartialModel>) -> Self {
		ActiveEmoteModel {
			id: value.id.into(),
			actor_id: value.added_by_id.map(Into::into),
			name: value.alias,
			timestamp: value.id.timestamp_ms() as i64,
			origin_id: value.origin_set_id.map(Into::into),
			flags: value.flags.into(),
			data,
		}
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote.model.go#L27
pub struct EmotePartialModel {
	pub id: EmoteId,
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

impl EmotePartialModel {
	pub fn from_db(value: Emote, owner: Option<UserPartialModel>, cdn_base_url: &str) -> Self {
		Self {
			id: value.id,
			name: value.default_name,
			animated: value.flags.contains(EmoteFlags::Animated),
			tags: value.tags,
			owner,
			state: EmoteVersionState::from_db(&value.flags),
			flags: value.flags.into(),
			lifecycle: if value.merged.is_some() {
				EmoteLifecycleModel::Deleted
			} else if value.image_set.input.is_pending() {
				EmoteLifecycleModel::Pending
			} else {
				EmoteLifecycleModel::Live
			},
			listed: value.flags.contains(EmoteFlags::PublicListed),
			host: ImageHost::from_image_set(&value.image_set, cdn_base_url),
		}
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote.model.go#L40
pub struct EmoteVersionModel {
	pub id: EmoteId,
	pub name: String,
	pub description: String,
	pub lifecycle: EmoteLifecycleModel,
	pub state: Vec<EmoteVersionState>,
	pub listed: bool,
	pub animated: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub host: Option<ImageHost>,
	#[serde(rename = "createdAt")]
	pub created_at: u64,
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

async_graphql::scalar!(EmoteLifecycleModel);

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote.model.go#L78
pub enum EmoteVersionState {
	#[default]
	Listed,
	Personal,
	NoPersonal,
}

impl EmoteVersionState {
	pub fn from_db(value: &EmoteFlags) -> Vec<Self> {
		let mut state = Vec::new();

		if value.contains(EmoteFlags::ApprovedPersonal) && !value.contains(EmoteFlags::DeniedPersonal) {
			state.push(Self::Personal);
		} else if value.contains(EmoteFlags::DeniedPersonal) {
			state.push(Self::NoPersonal);
		}

		if value.contains(EmoteFlags::PublicListed) {
			state.push(Self::Listed);
		}

		state
	}
}

async_graphql::scalar!(EmoteVersionState);

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
