use bitmask_enum::bitmask;

use crate::database::role::permissions::{
	AdminPermission, AllowDeny, BadgePermission, EmotePermission, EmoteSetPermission, PaintPermission, Permissions, PermissionsExt, RolePermission as NewRolePermissions, TicketPermission, UserPermission
};

#[bitmask(u64)]
// https://github.com/SevenTV/Common/blob/master/structures/v3/type.role.go#L37
pub enum RolePermission {
	CreateEmote = 1 << 0,
	EditEmote = 1 << 1,
	CreateEmoteSet = 1 << 2,
	EditEmoteSet = 1 << 3,

	CreateReport = 1 << 13,
	SendMessages = 1 << 14,

	FeatureZeroWidthEmoteType = 1 << 23,
	FeatureProfilePictureAnimation = 1 << 24,
	FeatureMessagingPriority = 1 << 25,

	ManageBans = 1 << 30,
	ManageRoles = 1 << 31,
	ManageReports = 1 << 32,
	ManageUsers = 1 << 33,

	EditAnyEmote = 1 << 41,
	EditAnyEmoteSet = 1 << 42,

	BypassPrivacy = 1 << 48,

	ManageContent = 1 << 54,
	ManageStack = 1 << 55,
	ManageCosmetics = 1 << 56,
	RunJobs = 1 << 57,
	ManageEntitlements = 1 << 58,

	SuperAdministrator = 1 << 62,
}

impl Default for RolePermission {
	fn default() -> Self {
		RolePermission::none()
	}
}

impl<'a> serde::Deserialize<'a> for RolePermission {
	fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
		let bits = u64::deserialize(deserializer)?;
		Ok(Self::from(bits))
	}
}

impl RolePermission {
	pub fn to_emote_permissions(allowed: Self, denied: Self) -> AllowDeny<EmotePermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::CreateEmote) {
			perm.allow(EmotePermission::Upload);
		}
		if allowed.contains(RolePermission::EditEmote) {
			perm.allow(EmotePermission::Edit);
			perm.allow(EmotePermission::Delete);
		}
		if allowed.contains(RolePermission::EditAnyEmote) {
			perm.allow(EmotePermission::Admin);
		}
		if denied.contains(RolePermission::CreateEmote) {
			perm.deny(EmotePermission::Upload);
		}
		if denied.contains(RolePermission::EditEmote) {
			perm.deny(EmotePermission::Edit);
			perm.deny(EmotePermission::Delete);
		}
		if denied.contains(RolePermission::EditAnyEmote) {
			perm.deny(EmotePermission::Admin);
		}

		perm
	}

	pub fn to_role_permissions(allowed: Self, denied: Self) -> AllowDeny<NewRolePermissions> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::ManageRoles) {
			perm.allow(NewRolePermissions::Admin);
		}
		if denied.contains(RolePermission::ManageRoles) {
			perm.deny(NewRolePermissions::Admin);
		}

		perm
	}

	pub fn to_emote_set_permission(allowed: Self, denied: Self) -> AllowDeny<EmoteSetPermission> {
		let mut perm = AllowDeny::default();

		if allowed.intersects(RolePermission::CreateEmoteSet | RolePermission::EditEmoteSet) {
			perm.allow(EmoteSetPermission::Manage);
		}
		if allowed.contains(RolePermission::EditAnyEmoteSet) {
			perm.allow(EmoteSetPermission::ManageAny);
		}
		if denied.intersects(RolePermission::CreateEmoteSet | RolePermission::EditEmoteSet) {
			perm.deny(EmoteSetPermission::Manage);
		}
		if denied.contains(RolePermission::EditAnyEmoteSet) {
			perm.deny(EmoteSetPermission::ManageAny);
		}

		perm
	}

	pub fn to_badge_permission(allowed: Self, denied: Self) -> AllowDeny<BadgePermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::ManageCosmetics) {
			perm.allow(BadgePermission::Admin);
		}
		if denied.contains(RolePermission::ManageCosmetics) {
			perm.deny(BadgePermission::Admin);
		}

		perm
	}

	pub fn to_paint_permission(allowed: Self, denied: Self) -> AllowDeny<PaintPermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::ManageCosmetics) {
			perm.allow(PaintPermission::Admin);
		}
		if denied.contains(RolePermission::ManageCosmetics) {
			perm.deny(PaintPermission::Admin);
		}

		perm
	}

	pub fn to_user_permission(allowed: Self, denied: Self) -> AllowDeny<UserPermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::ManageBans) {
			perm.allow(UserPermission::Moderate);
		}
		if allowed.contains(RolePermission::ManageUsers) {
			perm.allow(UserPermission::ManageAny);
		}
		if denied.contains(RolePermission::ManageBans) {
			perm.deny(UserPermission::Moderate);
		}
		if denied.contains(RolePermission::ManageUsers) {
			perm.deny(UserPermission::ManageAny);
		}

		if allowed.contains(RolePermission::FeatureProfilePictureAnimation) {
			perm.allow(UserPermission::UseCustomProfilePicture);
		}
		if denied.contains(RolePermission::FeatureProfilePictureAnimation) {
			perm.deny(UserPermission::UseCustomProfilePicture);
		}

		perm
	}

	pub fn to_ticket_permission(allowed: Self, denied: Self) -> AllowDeny<TicketPermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::CreateReport) {
			perm.allow(TicketPermission::Create);
		}
		if allowed.contains(RolePermission::ManageReports) {
			perm.allow(TicketPermission::Admin);
		}
		if denied.contains(RolePermission::CreateReport) {
			perm.deny(TicketPermission::Create);
		}
		if denied.contains(RolePermission::ManageReports) {
			perm.deny(TicketPermission::Admin);
		}

		perm
	}

	pub fn to_admin_permission(allowed: Self, denied: Self) -> AllowDeny<AdminPermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::SuperAdministrator) {
			perm.allow(AdminPermission::SuperAdmin);
		}
		if denied.contains(RolePermission::SuperAdministrator) {
			perm.deny(AdminPermission::SuperAdmin);
		}

		perm
	}

	pub fn to_new_permissions(allowed: Self, denied: Self) -> Permissions {
		Permissions {
			emote: Self::to_emote_permissions(allowed, denied),
			role: Self::to_role_permissions(allowed, denied),
			emote_set: Self::to_emote_set_permission(allowed, denied),
			badge: Self::to_badge_permission(allowed, denied),
			paint: Self::to_paint_permission(allowed, denied),
			user: Self::to_user_permission(allowed, denied),
			ticket: Self::to_ticket_permission(allowed, denied),
			admin: Self::to_admin_permission(allowed, denied),
			..Default::default()
		}
	}

	pub fn from_db(value: Permissions) -> (Self, Self) {
		let mut allowed = RolePermission::none();
		let mut denied = RolePermission::none();

		// Emote Permissions
		{
			if value.has(EmotePermission::Upload) {
				allowed |= Self::CreateEmote;
			}
			if value.has(EmotePermission::Edit) {
				allowed |= Self::EditEmote;
			}
			if value.has(EmotePermission::Admin) {
				allowed |= Self::EditAnyEmote;
			}

			if value.denied(EmotePermission::Upload) {
				denied |= Self::CreateEmote;
			}
			if value.denied(EmotePermission::Edit) {
				denied |= Self::EditEmote;
			}
			if value.denied(EmotePermission::Admin) {
				denied |= Self::EditAnyEmote;
			}
		}

		// Role Permissions
		{
			if value.has(NewRolePermissions::Admin) {
				allowed |= Self::ManageRoles;
			}
			if value.denied(NewRolePermissions::Admin) {
				denied |= Self::ManageRoles;
			}
		}

		// Emote Set Permissions
		{
			if value.has(EmoteSetPermission::Manage) {
				allowed |= Self::CreateEmoteSet | Self::EditEmoteSet;
			}
			if value.has(EmoteSetPermission::ManageAny) {
				allowed |= Self::EditAnyEmoteSet;
			}

			if value.denied(EmoteSetPermission::Manage) {
				denied |= Self::CreateEmoteSet | Self::EditEmoteSet;
			}
			if value.denied(EmoteSetPermission::ManageAny) {
				denied |= Self::EditAnyEmoteSet;
			}
		}

		// Cosmetics Permissions
		{
			if value.has_any([BadgePermission::Admin.into(), PaintPermission::Admin.into()]) {
				allowed |= Self::ManageCosmetics;
			}
			if value.denied_any([BadgePermission::Admin.into(), PaintPermission::Admin.into()]) {
				denied |= Self::ManageCosmetics;
			}
		}

		// User Permissions
		{
			if value.has(UserPermission::Moderate) {
				allowed |= Self::ManageBans;
			}
			if value.has(UserPermission::ManageAny) {
				allowed |= Self::ManageUsers;
			}

			if value.denied(UserPermission::Moderate) {
				denied |= Self::ManageBans;
			}
			if value.denied(UserPermission::ManageAny) {
				denied |= Self::ManageUsers;
			}

			if value.has(UserPermission::UseCustomProfilePicture) {
				allowed |= Self::FeatureProfilePictureAnimation;
			}
			if value.has(UserPermission::UseCustomProfilePicture) {
				denied |= Self::FeatureProfilePictureAnimation;
			}
		}

		// Ticket Permissions
		{
			if value.has(TicketPermission::Create) {
				allowed |= Self::CreateReport;
			}
			if value.has(TicketPermission::ManageAbuse) && value.has(TicketPermission::ManageGeneric) {
				allowed |= Self::ManageReports;
			}

			if value.denied(TicketPermission::Create) {
				denied |= Self::CreateReport;
			}
			if value.denied(TicketPermission::ManageAbuse) || value.denied(TicketPermission::ManageGeneric) {
				denied |= Self::ManageReports;
			}
		}

		// Admin Permissions
		{
			if value.has(AdminPermission::SuperAdmin) {
				allowed |= Self::SuperAdministrator;
			}
			if value.has(AdminPermission::SuperAdmin) {
				denied |= Self::SuperAdministrator;
			}
		}

		(allowed, denied)
	}
}
