//! This module contains types that are only used in the API but are needed for
//! both REST and GraphQL endpoints.

use bitmask_enum::bitmask;
use shared::database::{EmoteFlags, EmotePermission, EmoteSetPermission, UserEditorPermissions};

#[derive(utoipa::OpenApi)]
#[openapi(components(schemas(
	// Emote
	EmoteLifecycleModel,
	EmoteVersionState,
    // User
    UserEditorModelPermission,
)))]
pub struct Docs;

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
	AllowPersonal,
	NoPersonal,
}

impl EmoteVersionState {
	pub fn from_db(value: &EmoteFlags) -> Vec<Self> {
		let mut state = Vec::new();

		if value.contains(EmoteFlags::ApprovedPersonal) && !value.contains(EmoteFlags::DeniedPersonal) {
			state.push(Self::AllowPersonal);
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

		if value.has_emote_set(EmoteSetPermission::Edit) {
			perms |= Self::ManageEmoteSets;
		}

		if value.has_emote(EmotePermission::Edit) {
			perms |= Self::ManageOwnedEmotes;
		}

		perms
	}

	pub fn to_db(&self) -> UserEditorPermissions {
		let mut perms = UserEditorPermissions::default();

		if self.contains(Self::ManageEmoteSets) {
			perms.emote_set.allow(EmoteSetPermission::Edit);
		}

		if self.contains(Self::ManageOwnedEmotes) {
			perms.emote.allow(EmotePermission::Edit);
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
