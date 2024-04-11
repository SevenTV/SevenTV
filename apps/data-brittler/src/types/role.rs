use bitmask_enum::bitmask;
use shared::database::{self, AllowDeny};
use shared::object_id::ObjectId;

// https://github.com/SevenTV/Common/blob/master/structures/v3/type.role.go

#[derive(Debug, serde::Deserialize)]
pub struct Role {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub name: String,
	pub position: u32,
	pub color: i32,
	#[serde(default)]
	pub allowed: RolePermission,
	#[serde(default)]
	pub denied: RolePermission,
	pub discord_id: Option<u64>,
}

#[bitmask(i64)]
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
	fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<RolePermission, D::Error> {
		let bits = i64::deserialize(deserializer)?;
		Ok(RolePermission::from(bits))
	}
}

impl Role {
	pub fn to_emote_permissions(&self) -> AllowDeny<database::EmotePermission> {
		let mut perm = AllowDeny::default();

		if self.allowed.contains(RolePermission::CreateEmote) {
			perm.allow(database::EmotePermission::Upload);
		}
		if self.allowed.contains(RolePermission::EditEmote) {
			perm.allow(database::EmotePermission::Edit);
			perm.allow(database::EmotePermission::Delete);
		}
		if self.allowed.contains(RolePermission::EditAnyEmote) {
			perm.allow(database::EmotePermission::Admin);
		}
		if self.denied.contains(RolePermission::CreateEmote) {
			perm.deny(database::EmotePermission::Upload);
		}
		if self.denied.contains(RolePermission::EditEmote) {
			perm.deny(database::EmotePermission::Edit);
			perm.deny(database::EmotePermission::Delete);
		}
		if self.denied.contains(RolePermission::EditAnyEmote) {
			perm.deny(database::EmotePermission::Admin);
		}

		perm
	}

	pub fn to_role_permissions(&self) -> AllowDeny<database::RolePermission> {
		let mut perm = AllowDeny::default();

		if self.allowed.contains(RolePermission::ManageRoles) {
			perm.allow(database::RolePermission::Admin);
		}
		if self.denied.contains(RolePermission::ManageRoles) {
			perm.deny(database::RolePermission::Admin);
		}

		perm
	}

	pub fn to_emote_set_permission(&self) -> AllowDeny<database::EmoteSetPermission> {
		let mut perm = AllowDeny::default();

		if self.allowed.contains(RolePermission::CreateEmoteSet) {
			perm.allow(database::EmoteSetPermission::Create);
		}
		if self.allowed.contains(RolePermission::EditEmoteSet) {
			perm.allow(database::EmoteSetPermission::Edit);
			perm.allow(database::EmoteSetPermission::Delete);
		}
		if self.allowed.contains(RolePermission::EditAnyEmoteSet) {
			perm.allow(database::EmoteSetPermission::Admin);
		}
		if self.denied.contains(RolePermission::CreateEmoteSet) {
			perm.deny(database::EmoteSetPermission::Create);
		}
		if self.denied.contains(RolePermission::EditEmoteSet) {
			perm.deny(database::EmoteSetPermission::Edit);
			perm.deny(database::EmoteSetPermission::Delete);
		}
		if self.denied.contains(RolePermission::EditAnyEmoteSet) {
			perm.deny(database::EmoteSetPermission::Admin);
		}

		perm
	}

	pub fn to_badge_permission(&self) -> AllowDeny<database::BadgePermission> {
		let mut perm = AllowDeny::default();

		if self.allowed.contains(RolePermission::ManageCosmetics) {
			perm.allow(database::BadgePermission::Admin);
		}
		if self.denied.contains(RolePermission::ManageCosmetics) {
			perm.deny(database::BadgePermission::Admin);
		}

		perm
	}

	pub fn to_paint_permission(&self) -> AllowDeny<database::PaintPermission> {
		let mut perm = AllowDeny::default();

		if self.allowed.contains(RolePermission::ManageCosmetics) {
			perm.allow(database::PaintPermission::Admin);
		}
		if self.denied.contains(RolePermission::ManageCosmetics) {
			perm.deny(database::PaintPermission::Admin);
		}

		perm
	}

	pub fn to_user_permission(&self) -> database::AllowDeny<database::UserPermission> {
		let mut perm = AllowDeny::default();

		if self.allowed.contains(RolePermission::ManageBans) {
			perm.allow(database::UserPermission::Ban);
		}
		if self.allowed.contains(RolePermission::ManageUsers) {
			perm.allow(database::UserPermission::Admin);
		}
		if self.denied.contains(RolePermission::ManageBans) {
			perm.deny(database::UserPermission::Ban);
		}
		if self.denied.contains(RolePermission::ManageUsers) {
			perm.deny(database::UserPermission::Admin);
		}

		perm
	}

	pub fn to_feature_permission(&self) -> database::AllowDeny<database::FeaturePermission> {
		let mut perm = AllowDeny::default();

		if self.allowed.contains(RolePermission::FeatureProfilePictureAnimation) {
			perm.allow(database::FeaturePermission::AnimatedProfilePicture);
		}
		if self.denied.contains(RolePermission::FeatureProfilePictureAnimation) {
			perm.deny(database::FeaturePermission::AnimatedProfilePicture);
		}

		perm
	}

	pub fn to_admin_permission(&self) -> database::AllowDeny<database::AdminPermission> {
		let mut perm = AllowDeny::default();

		if self.allowed.contains(RolePermission::SuperAdministrator) {
			perm.allow(database::AdminPermission::SuperAdmin);
		}
		if self.denied.contains(RolePermission::SuperAdministrator) {
			perm.deny(database::AdminPermission::SuperAdmin);
		}

		perm
	}

	pub fn to_new_permissions(&self) -> database::Permissions {
		database::Permissions {
			emote: self.to_emote_permissions(),
			role: self.to_role_permissions(),
			emote_set: self.to_emote_set_permission(),
			badge: self.to_badge_permission(),
			paint: self.to_paint_permission(),
			user: self.to_user_permission(),
			feature: self.to_feature_permission(),
			admin: self.to_admin_permission(),
			..Default::default()
		}
	}
}
