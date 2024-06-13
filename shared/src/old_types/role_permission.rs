use bitmask_enum::bitmask;

use crate::database::{self, AllowDeny};

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
	pub fn to_emote_permissions(allowed: Self, denied: Self) -> AllowDeny<database::EmotePermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::CreateEmote) {
			perm.allow(database::EmotePermission::Upload);
		}
		if allowed.contains(RolePermission::EditEmote) {
			perm.allow(database::EmotePermission::Edit);
			perm.allow(database::EmotePermission::Delete);
		}
		if allowed.contains(RolePermission::EditAnyEmote) {
			perm.allow(database::EmotePermission::Admin);
		}
		if denied.contains(RolePermission::CreateEmote) {
			perm.deny(database::EmotePermission::Upload);
		}
		if denied.contains(RolePermission::EditEmote) {
			perm.deny(database::EmotePermission::Edit);
			perm.deny(database::EmotePermission::Delete);
		}
		if denied.contains(RolePermission::EditAnyEmote) {
			perm.deny(database::EmotePermission::Admin);
		}

		perm
	}

	pub fn to_role_permissions(allowed: Self, denied: Self) -> AllowDeny<database::RolePermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::ManageRoles) {
			perm.allow(database::RolePermission::Admin);
		}
		if denied.contains(RolePermission::ManageRoles) {
			perm.deny(database::RolePermission::Admin);
		}

		perm
	}

	pub fn to_emote_set_permission(allowed: Self, denied: Self) -> AllowDeny<database::EmoteSetPermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::CreateEmoteSet) {
			perm.allow(database::EmoteSetPermission::Create);
		}
		if allowed.contains(RolePermission::EditEmoteSet) {
			perm.allow(database::EmoteSetPermission::Edit);
			perm.allow(database::EmoteSetPermission::Delete);
		}
		if allowed.contains(RolePermission::EditAnyEmoteSet) {
			perm.allow(database::EmoteSetPermission::Admin);
		}
		if denied.contains(RolePermission::CreateEmoteSet) {
			perm.deny(database::EmoteSetPermission::Create);
		}
		if denied.contains(RolePermission::EditEmoteSet) {
			perm.deny(database::EmoteSetPermission::Edit);
			perm.deny(database::EmoteSetPermission::Delete);
		}
		if denied.contains(RolePermission::EditAnyEmoteSet) {
			perm.deny(database::EmoteSetPermission::Admin);
		}

		perm
	}

	pub fn to_badge_permission(allowed: Self, denied: Self) -> AllowDeny<database::BadgePermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::ManageCosmetics) {
			perm.allow(database::BadgePermission::Admin);
		}
		if denied.contains(RolePermission::ManageCosmetics) {
			perm.deny(database::BadgePermission::Admin);
		}

		perm
	}

	pub fn to_paint_permission(allowed: Self, denied: Self) -> AllowDeny<database::PaintPermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::ManageCosmetics) {
			perm.allow(database::PaintPermission::Admin);
		}
		if denied.contains(RolePermission::ManageCosmetics) {
			perm.deny(database::PaintPermission::Admin);
		}

		perm
	}

	pub fn to_user_permission(allowed: Self, denied: Self) -> database::AllowDeny<database::UserPermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::ManageBans) {
			perm.allow(database::UserPermission::Ban);
		}
		if allowed.contains(RolePermission::ManageUsers) {
			perm.allow(database::UserPermission::Admin);
		}
		if denied.contains(RolePermission::ManageBans) {
			perm.deny(database::UserPermission::Ban);
		}
		if denied.contains(RolePermission::ManageUsers) {
			perm.deny(database::UserPermission::Admin);
		}

		perm
	}

	pub fn to_feature_permission(allowed: Self, denied: Self) -> database::AllowDeny<database::FeaturePermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::FeatureProfilePictureAnimation) {
			perm.allow(database::FeaturePermission::UseCustomProfilePicture);
		}
		if denied.contains(RolePermission::FeatureProfilePictureAnimation) {
			perm.deny(database::FeaturePermission::UseCustomProfilePicture);
		}

		perm
	}

	pub fn to_ticket_permission(allowed: Self, denied: Self) -> database::AllowDeny<database::TicketPermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::CreateReport) {
			perm.allow(database::TicketPermission::Create);
		}
		if allowed.contains(RolePermission::ManageReports) {
			perm.allow(database::TicketPermission::Admin);
		}
		if denied.contains(RolePermission::CreateReport) {
			perm.deny(database::TicketPermission::Create);
		}
		if denied.contains(RolePermission::ManageReports) {
			perm.deny(database::TicketPermission::Admin);
		}

		perm
	}

	pub fn to_admin_permission(allowed: Self, denied: Self) -> database::AllowDeny<database::AdminPermission> {
		let mut perm = AllowDeny::default();

		if allowed.contains(RolePermission::SuperAdministrator) {
			perm.allow(database::AdminPermission::SuperAdmin);
		}
		if denied.contains(RolePermission::SuperAdministrator) {
			perm.deny(database::AdminPermission::SuperAdmin);
		}

		perm
	}

	pub fn to_new_permissions(allowed: Self, denied: Self) -> database::Permissions {
		database::Permissions {
			emote: Self::to_emote_permissions(allowed, denied),
			role: Self::to_role_permissions(allowed, denied),
			emote_set: Self::to_emote_set_permission(allowed, denied),
			badge: Self::to_badge_permission(allowed, denied),
			paint: Self::to_paint_permission(allowed, denied),
			user: Self::to_user_permission(allowed, denied),
			feature: Self::to_feature_permission(allowed, denied),
			ticket: Self::to_ticket_permission(allowed, denied),
			admin: Self::to_admin_permission(allowed, denied),
			..Default::default()
		}
	}

	pub fn from_db(value: database::Permissions) -> (Self, Self) {
		let mut allowed = RolePermission::none();
		let mut denied = RolePermission::none();

		// Emote Permissions
		{
			if value.emote.allow.contains(database::EmotePermission::Upload) {
				allowed |= Self::CreateEmote;
			}
			if value.emote.allow.contains(database::EmotePermission::Edit) {
				allowed |= Self::EditEmote;
			}
			if value.emote.allow.contains(database::EmotePermission::Admin) {
				allowed |= Self::EditAnyEmote;
			}

			if value.emote.deny.contains(database::EmotePermission::Upload) {
				denied |= Self::CreateEmote;
			}
			if value.emote.deny.contains(database::EmotePermission::Edit) {
				denied |= Self::EditEmote;
			}
			if value.emote.deny.contains(database::EmotePermission::Admin) {
				denied |= Self::EditAnyEmote;
			}
		}

		// Role Permissions
		{
			if value.role.allow.contains(database::RolePermission::Admin) {
				allowed |= Self::ManageRoles;
			}
			if value.role.deny.contains(database::RolePermission::Admin) {
				denied |= Self::ManageRoles;
			}
		}

		// Emote Set Permissions
		{
			if value.emote_set.allow.contains(database::EmoteSetPermission::Create) {
				allowed |= Self::CreateEmoteSet;
			}
			if value.emote_set.allow.contains(database::EmoteSetPermission::Edit) {
				allowed |= Self::EditEmoteSet;
			}
			if value.emote_set.allow.contains(database::EmoteSetPermission::Admin) {
				allowed |= Self::EditAnyEmoteSet;
			}

			if value.emote_set.deny.contains(database::EmoteSetPermission::Create) {
				denied |= Self::CreateEmoteSet;
			}
			if value.emote_set.deny.contains(database::EmoteSetPermission::Edit) {
				denied |= Self::EditEmoteSet;
			}
			if value.emote_set.deny.contains(database::EmoteSetPermission::Admin) {
				denied |= Self::EditAnyEmoteSet;
			}
		}

		// Cosmetics Permissions
		{
			if value.badge.allow.contains(database::BadgePermission::Admin)
				&& value.paint.allow.contains(database::PaintPermission::Admin)
			{
				allowed |= Self::ManageCosmetics;
			}

			if value.badge.deny.contains(database::BadgePermission::Admin)
				&& value.paint.deny.contains(database::PaintPermission::Admin)
			{
				denied |= Self::ManageCosmetics;
			}
		}

		// User Permissions
		{
			if value.user.allow.contains(database::UserPermission::Ban) {
				allowed |= Self::ManageBans;
			}
			if value.user.allow.contains(database::UserPermission::Admin) {
				allowed |= Self::ManageUsers;
			}

			if value.user.deny.contains(database::UserPermission::Ban) {
				denied |= Self::ManageBans;
			}
			if value.user.deny.contains(database::UserPermission::Admin) {
				denied |= Self::ManageUsers;
			}
		}

		// Feature Permissions
		{
			if value
				.feature
				.allow
				.contains(database::FeaturePermission::UseCustomProfilePicture)
			{
				allowed |= Self::FeatureProfilePictureAnimation;
			}

			if value
				.feature
				.deny
				.contains(database::FeaturePermission::UseCustomProfilePicture)
			{
				denied |= Self::FeatureProfilePictureAnimation;
			}
		}

		// Report Permissions
		{
			if value.ticket.allow.contains(database::TicketPermission::Create) {
				allowed |= Self::CreateReport;
			}
			if value.ticket.allow.contains(database::TicketPermission::Edit) {
				allowed |= Self::ManageReports;
			}

			if value.ticket.deny.contains(database::TicketPermission::Create) {
				denied |= Self::CreateReport;
			}
			if value.ticket.deny.contains(database::TicketPermission::Edit) {
				denied |= Self::ManageReports;
			}
		}

		// Admin Permissions
		{
			if value.admin.allow.contains(database::AdminPermission::SuperAdmin) {
				allowed |= Self::SuperAdministrator;
			}

			if value.admin.deny.contains(database::AdminPermission::SuperAdmin) {
				denied |= Self::SuperAdministrator;
			}
		}

		(allowed, denied)
	}
}
