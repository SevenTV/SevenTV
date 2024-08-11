mod emote;
mod entitlement;
mod role;
mod user_presence;

pub use emote::*;
pub use entitlement::*;
pub use role::*;
use shared::old_types::{
	ActiveEmoteModel, EmoteLifecycleModel, EmotePartialModel, EmoteSetModel, EmoteSetOrigin, EmoteSetPartialModel,
	EmoteVersionModel, EmoteVersionState, UserConnectionModel, UserEditorModel, UserEditorModelPermission, UserModel,
};
pub use user_presence::*;

#[derive(utoipa::OpenApi)]
#[openapi(components(schemas(
	// Emote
	EmoteModel,
	EmotePartialModel,
	EmoteVersionModel,
	EmoteLifecycleModel,
	EmoteVersionState,
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
    UserEditorModelPermission,
)))]
pub struct Docs;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
	value == &T::default()
}
