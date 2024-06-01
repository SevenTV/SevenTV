//! This module contains types that are only used in the REST API.

mod emote;
mod emote_set;
mod entitlement;
mod role;
mod user;
mod user_connection;
mod user_presence;

pub use emote::*;
pub use emote_set::*;
pub use entitlement::*;
pub use role::*;
pub use user::*;
pub use user_connection::*;
pub use user_presence::*;

#[derive(utoipa::OpenApi)]
#[openapi(components(schemas(
	// Emote
	EmoteModel,
	EmotePartialModel,
	EmoteVersionModel,
	// Emote Set
	EmoteSetModel,
	EmoteSetPartialModel,
	ActiveEmoteModel,
	EmoteSetOrigin,
	// Entitlement
	EntitlementModel,
	EntitlementKind,
	// Role
	RoleModel,
	// UserConnection
	UserConnectionModel,
	// UserPresence
	PresenceModel,
	PresenceKind,
	UserPresenceWriteResponse,
	// User
	UserModel,
	UserEditorModel,
)))]
pub struct Docs;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
	value == &T::default()
}
