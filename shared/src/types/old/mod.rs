mod cosmetic;
mod emote;
mod emote_set;
mod entitlement;
mod image;
mod message;
mod role;
mod user;
mod user_connection;
mod user_presence;

pub use cosmetic::*;
pub use emote::*;
pub use emote_set::*;
pub use entitlement::*;
pub use image::*;
pub use message::*;
pub use role::*;
pub use user::*;
pub use user_connection::*;
pub use user_presence::*;

#[derive(utoipa::OpenApi)]
#[openapi(components(schemas(
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
	// Emote
	EmoteModel,
	EmotePartialModel,
	EmoteVersionModel,
	EmoteLifecycleModel,
	EmoteFlagsModel,
	EmoteVersionState,
	// Emote Set
	EmoteSetModel,
	EmoteSetPartialModel,
	EmoteSetFlagModel,
	ActiveEmoteModel,
	ActiveEmoteFlagModel,
	EmoteSetOrigin,
	// Entitlement
	EntitlementModel,
	EntitlementKind,
	// Image
	ImageHost,
	ImageFile,
	ImageFormat,
	// Role
	RoleModel,
	// UserConnection
	UserConnectionModel,
	UserConnectionPartialModel,
	UserConnectionPlatformModel,
	// UserPresence
	PresenceModel,
	PresenceKind,
	UserPresenceWriteResponse,
	// User
	UserModel,
	UserPartialModel,
	UserStyle,
	UserTypeModel,
	UserEditorModel,
)))]
pub struct Docs;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
	value == &T::default()
}
