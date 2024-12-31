use std::collections::HashMap;

use shared::database::role::permissions::{self, PermissionsExt, RateLimits};

#[derive(async_graphql::SimpleObject)]
pub struct Permissions {
	pub emote: EmotePermission,
	pub role: RolePermission,
	pub emote_set: EmoteSetPermission,
	pub badge: BadgePermission,
	pub paint: PaintPermission,
	pub flags: FlagPermission,
	pub user: UserPermission,
	pub ticket: TicketPermission,
	pub emote_moderation_request: EmoteModerationRequestPermission,
	pub admin: AdminPermission,
	pub emote_moderation_request_priority: Option<i32>,
	pub emote_moderation_request_limit: Option<i32>,
	pub emote_set_limit: Option<i32>,
	pub emote_set_capacity: Option<i32>,
	pub personal_emote_set_capacity: Option<i32>,
	pub ratelimits: HashMap<String, Option<RateLimits>>,
}

impl From<permissions::Permissions> for Permissions {
	fn from(permissions: permissions::Permissions) -> Self {
		Self {
			emote: EmotePermission::from_db(&permissions),
			role: RolePermission::from_db(&permissions),
			emote_set: EmoteSetPermission::from_db(&permissions),
			badge: BadgePermission::from_db(&permissions),
			paint: PaintPermission::from_db(&permissions),
			flags: FlagPermission::from_db(&permissions),
			user: UserPermission::from_db(&permissions),
			ticket: TicketPermission::from_db(&permissions),
			emote_moderation_request: EmoteModerationRequestPermission::from_db(&permissions),
			admin: AdminPermission::from_db(&permissions),
			emote_moderation_request_priority: permissions.emote_moderation_request_priority,
			emote_moderation_request_limit: permissions.emote_moderation_request_limit,
			emote_set_limit: permissions.emote_set_limit,
			emote_set_capacity: permissions.emote_set_capacity,
			personal_emote_set_capacity: permissions.personal_emote_set_capacity,
			ratelimits: permissions.ratelimits,
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct EmotePermission {
	pub admin: bool,
	pub upload: bool,
	pub delete: bool,
	pub edit: bool,
	pub manage_any: bool,
	pub merge: bool,
	pub view_unlisted: bool,
}

impl EmotePermission {
	fn from_db<T: PermissionsExt>(permissions: &T) -> Self {
		Self {
			admin: permissions.has(permissions::EmotePermission::Admin),
			upload: permissions.has(permissions::EmotePermission::Upload),
			delete: permissions.has(permissions::EmotePermission::Delete),
			edit: permissions.has(permissions::EmotePermission::Edit),
			manage_any: permissions.has(permissions::EmotePermission::ManageAny),
			merge: permissions.has(permissions::EmotePermission::Merge),
			view_unlisted: permissions.has(permissions::EmotePermission::ViewUnlisted),
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct RolePermission {
	pub admin: bool,
	pub manage: bool,
	pub assign: bool,
}

impl RolePermission {
	fn from_db<T: PermissionsExt>(permissions: &T) -> Self {
		Self {
			admin: permissions.has(permissions::RolePermission::Admin),
			manage: permissions.has(permissions::RolePermission::Manage),
			assign: permissions.has(permissions::RolePermission::Assign),
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct EmoteSetPermission {
	pub admin: bool,
	pub manage: bool,
	pub manage_any: bool,
	pub resize: bool,
	pub manage_global: bool,
	pub manage_special: bool,
	pub assign: bool,
}

impl EmoteSetPermission {
	fn from_db<T: PermissionsExt>(permissions: &T) -> Self {
		Self {
			admin: permissions.has(permissions::EmoteSetPermission::Admin),
			manage: permissions.has(permissions::EmoteSetPermission::Manage),
			manage_any: permissions.has(permissions::EmoteSetPermission::ManageAny),
			resize: permissions.has(permissions::EmoteSetPermission::Resize),
			manage_global: permissions.has(permissions::EmoteSetPermission::ManageGlobal),
			manage_special: permissions.has(permissions::EmoteSetPermission::ManageSpecial),
			assign: permissions.has(permissions::EmoteSetPermission::Assign),
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct BadgePermission {
	pub admin: bool,
	pub manage: bool,
	pub assign: bool,
}

impl BadgePermission {
	fn from_db<T: PermissionsExt>(permissions: &T) -> Self {
		Self {
			admin: permissions.has(permissions::BadgePermission::Admin),
			manage: permissions.has(permissions::BadgePermission::Manage),
			assign: permissions.has(permissions::BadgePermission::Assign),
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct PaintPermission {
	pub admin: bool,
	pub manage: bool,
	pub assign: bool,
}

impl PaintPermission {
	fn from_db<T: PermissionsExt>(permissions: &T) -> Self {
		Self {
			admin: permissions.has(permissions::PaintPermission::Admin),
			manage: permissions.has(permissions::PaintPermission::Manage),
			assign: permissions.has(permissions::PaintPermission::Assign),
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct FlagPermission {
	pub hidden: bool,
}

impl FlagPermission {
	fn from_db<T: PermissionsExt>(permissions: &T) -> Self {
		Self {
			hidden: permissions.has(permissions::FlagPermission::Hidden),
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct UserPermission {
	pub admin: bool,
	pub login: bool,
	pub invite_editors: bool,
	pub use_custom_profile_picture: bool,
	pub use_personal_emote_set: bool,
	pub use_badge: bool,
	pub use_paint: bool,
	pub manage_any: bool,
	pub billing: bool,
	pub manage_billing: bool,
	pub moderate: bool,
	pub view_hidden: bool,
	pub manage_sessions: bool,
}

impl UserPermission {
	fn from_db<T: PermissionsExt>(permissions: &T) -> Self {
		Self {
			admin: permissions.has(permissions::UserPermission::Admin),
			login: permissions.has(permissions::UserPermission::Login),
			invite_editors: permissions.has(permissions::UserPermission::InviteEditors),
			use_custom_profile_picture: permissions.has(permissions::UserPermission::UseCustomProfilePicture),
			use_personal_emote_set: permissions.has(permissions::UserPermission::UsePersonalEmoteSet),
			use_badge: permissions.has(permissions::UserPermission::UseBadge),
			use_paint: permissions.has(permissions::UserPermission::UsePaint),
			manage_any: permissions.has(permissions::UserPermission::ManageAny),
			billing: permissions.has(permissions::UserPermission::Billing),
			manage_billing: permissions.has(permissions::UserPermission::ManageBilling),
			moderate: permissions.has(permissions::UserPermission::Moderate),
			view_hidden: permissions.has(permissions::UserPermission::ViewHidden),
			manage_sessions: permissions.has(permissions::UserPermission::ManageSessions),
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct TicketPermission {
	pub admin: bool,
	pub create: bool,
	pub manage_abuse: bool,
	pub manage_billing: bool,
	pub manage_generic: bool,
	pub message: bool,
}

impl TicketPermission {
	fn from_db<T: PermissionsExt>(permissions: &T) -> Self {
		Self {
			admin: permissions.has(permissions::TicketPermission::Admin),
			create: permissions.has(permissions::TicketPermission::Create),
			manage_abuse: permissions.has(permissions::TicketPermission::ManageAbuse),
			manage_billing: permissions.has(permissions::TicketPermission::ManageBilling),
			manage_generic: permissions.has(permissions::TicketPermission::ManageGeneric),
			message: permissions.has(permissions::TicketPermission::Message),
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct EmoteModerationRequestPermission {
	pub admin: bool,
	pub manage: bool,
}

impl EmoteModerationRequestPermission {
	fn from_db<T: PermissionsExt>(permissions: &T) -> Self {
		Self {
			admin: permissions.has(permissions::EmoteModerationRequestPermission::Admin),
			manage: permissions.has(permissions::EmoteModerationRequestPermission::Manage),
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct AdminPermission {
	pub admin: bool,
	pub super_admin: bool,
	pub bypass_rate_limit: bool,
	pub manage_redeem_codes: bool,
	pub manage_entitlements: bool,
}

impl AdminPermission {
	fn from_db<T: PermissionsExt>(permissions: &T) -> Self {
		Self {
			admin: permissions.has(permissions::AdminPermission::Admin),
			super_admin: permissions.has(permissions::AdminPermission::SuperAdmin),
			bypass_rate_limit: permissions.has(permissions::AdminPermission::BypassRateLimit),
			manage_redeem_codes: permissions.has(permissions::AdminPermission::ManageRedeemCodes),
			manage_entitlements: permissions.has(permissions::AdminPermission::ManageEntitlements),
		}
	}
}
