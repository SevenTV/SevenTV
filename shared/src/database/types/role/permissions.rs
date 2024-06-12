use std::collections::HashMap;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use bitmask_enum::bitmask;
use enum_impl::EnumImpl;

pub trait BitMask:
	BitAnd<Output = Self>
	+ BitOr<Output = Self>
	+ Not<Output = Self>
	+ Not<Output = Self>
	+ BitOrAssign
	+ BitAndAssign
	+ Copy
	+ Default
	+ PartialEq
	+ Sized
	+ From<Self::Bits>
{
	type Bits: Copy + serde::Serialize + serde::de::DeserializeOwned;

	fn bits(&self) -> Self::Bits;

	fn is_default(&self) -> bool {
		*self == Self::default()
	}

	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serde::Serialize::serialize(&self.bits(), serializer)
	}

	fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		use serde::de::Deserialize;

		let value = Self::Bits::deserialize(deserializer)?;
		Ok(Self::from(value))
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default, Copy, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[serde(bound(serialize = "T: BitMask", deserialize = "T: BitMask"))]
pub struct AllowDeny<T: BitMask> {
	#[serde(skip_serializing_if = "T::is_default")]
	#[serde(serialize_with = "T::serialize")]
	#[serde(deserialize_with = "T::deserialize")]
	#[serde(default)]
	pub allow: T,
	#[serde(skip_serializing_if = "T::is_default")]
	#[serde(serialize_with = "T::serialize")]
	#[serde(deserialize_with = "T::deserialize")]
	#[serde(default)]
	pub deny: T,
}

impl<T: BitMask> AllowDeny<T> {
	pub fn permission(&self) -> T {
		self.allow & !self.deny
	}

	pub fn merge(&mut self, other: Self) {
		self.allow(other.allow);
		self.deny(other.deny);
	}

	pub fn allow(&mut self, permission: T) {
		self.allow |= permission;
		self.deny &= !permission;
	}

	pub fn deny(&mut self, permission: T) {
		self.allow &= !permission;
		self.deny |= permission;
	}

	pub fn is_empty(&self) -> bool {
		self.allow == T::default() && self.deny == T::default()
	}
}

#[bitmask(u8)]
pub enum EmotePermission {
	Upload = 1,
	Delete = 2,
	Edit = 4,
	Admin = 8,
}

impl Default for EmotePermission {
	fn default() -> Self {
		Self::none()
	}
}

impl BitMask for EmotePermission {
	type Bits = u8;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

#[bitmask(u8)]
pub enum RolePermission {
	Create = 1,
	Delete = 2,
	Edit = 4,
	Assign = 8,
	Admin = 16,
}

impl BitMask for RolePermission {
	type Bits = u8;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for RolePermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(u8)]
pub enum EmoteSetPermission {
	Create = 1,
	Delete = 2,
	Edit = 4,
	Admin = 8,
}

impl BitMask for EmoteSetPermission {
	type Bits = u8;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for EmoteSetPermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(u8)]
pub enum BadgePermission {
	Create = 1,
	Delete = 2,
	Edit = 4,
	Admin = 8,
}

impl BitMask for BadgePermission {
	type Bits = u8;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for BadgePermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(u8)]
pub enum PaintPermission {
	Create = 1,
	Delete = 2,
	Edit = 4,
	Admin = 8,
}

impl BitMask for PaintPermission {
	type Bits = u8;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for PaintPermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(u8)]
pub enum UserPermission {
	Ban = 1,
	Merge = 2,
	Login = 4,
	Delete = 8,
	Edit = 16,
	Admin = 32,
}

impl BitMask for UserPermission {
	type Bits = u8;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for UserPermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(u16)]
pub enum FeaturePermission {
	UseCustomProfilePicture = 1,
	PersonalEmoteSet = 2,
	UseBadge = 4,
	BypassEmoteSetCountLimit = 8,
	ByPassEmoteSetSlotsLimit = 16,
	ByPassPersonalEmoteSetSlotsLimit = 32,
	Admin = 64,
	UsePaint = 128,
	UsePersonalEmoteSet = 256,
}

impl BitMask for FeaturePermission {
	type Bits = u16;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for FeaturePermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(u8)]
pub enum TicketPermission {
	Create = 1,
	Read = 2,
	Message = 4,
	Edit = 8,
	Admin = 16,
}

impl BitMask for TicketPermission {
	type Bits = u8;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for TicketPermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(u8)]
pub enum AdminPermission {
	Admin = 1,
	SuperAdmin = 2,
}

impl BitMask for AdminPermission {
	type Bits = u8;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for AdminPermission {
	fn default() -> Self {
		Self::none()
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Permissions {
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub emote: AllowDeny<EmotePermission>,
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub role: AllowDeny<RolePermission>,
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub emote_set: AllowDeny<EmoteSetPermission>,
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub badge: AllowDeny<BadgePermission>,
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub paint: AllowDeny<PaintPermission>,
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub user: AllowDeny<UserPermission>,
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub feature: AllowDeny<FeaturePermission>,
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub ticket: AllowDeny<TicketPermission>,
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub admin: AllowDeny<AdminPermission>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub emote_set_count_limit: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub emote_set_slots_limit: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub personal_emote_set_slots_limit: Option<u16>,

	#[serde(flatten)]
	pub unknown: HashMap<String, serde_json::Value>,
}

impl Permissions {
	pub fn merge(&mut self, other: Self) {
		self.merge_ref(&other);
		self.unknown.extend(other.unknown);
	}

	pub fn merge_ref(&mut self, other: &Self) {
		self.emote.merge(other.emote);
		self.role.merge(other.role);
		self.emote_set.merge(other.emote_set);
		self.badge.merge(other.badge);
		self.paint.merge(other.paint);
		self.user.merge(other.user);
		self.feature.merge(other.feature);
		self.ticket.merge(other.ticket);
		self.admin.merge(other.admin);

		self.emote_set_count_limit = other.emote_set_count_limit.or(self.emote_set_count_limit);
		self.emote_set_slots_limit = other.emote_set_slots_limit.or(self.emote_set_slots_limit);
		self.personal_emote_set_slots_limit = other.personal_emote_set_slots_limit.or(self.personal_emote_set_slots_limit);
	}

	pub fn apply(&mut self, perm: Permission) {
		match perm {
			Permission::Emote(perm) => self.emote.allow(perm),
			Permission::Role(perm) => self.role.allow(perm),
			Permission::EmoteSet(perm) => self.emote_set.allow(perm),
			Permission::Badge(perm) => self.badge.allow(perm),
			Permission::Paint(perm) => self.paint.allow(perm),
			Permission::User(perm) => self.user.allow(perm),
			Permission::Feature(perm) => self.feature.allow(perm),
			Permission::Ticket(perm) => self.ticket.allow(perm),
			Permission::Admin(perm) => self.admin.allow(perm),

			Permission::EmoteSetCount(perm) => self.emote_set_count_limit = Some(perm),
			Permission::EmoteSetSlots(perm) => self.emote_set_slots_limit = Some(perm),
			Permission::PersonalEmoteSetSlots(perm) => self.personal_emote_set_slots_limit = Some(perm),
		}
	}

	pub fn has(&self, permission: impl Into<Permission>) -> bool {
		self.has_permission(permission)
	}

	pub fn has_emote(&self, permission: EmotePermission) -> bool {
		self.is_admin()
			|| self.emote.permission().contains(permission)
			|| self.emote.permission().contains(EmotePermission::Admin)
	}

	pub fn has_role(&self, permission: RolePermission) -> bool {
		self.is_admin()
			|| self.role.permission().contains(permission)
			|| self.role.permission().contains(RolePermission::Admin)
	}

	pub fn has_emote_set(&self, permission: EmoteSetPermission) -> bool {
		self.is_admin()
			|| self.emote_set.permission().contains(permission)
			|| self.emote_set.permission().contains(EmoteSetPermission::Admin)
	}

	pub fn has_badge(&self, permission: BadgePermission) -> bool {
		self.is_admin()
			|| self.badge.permission().contains(permission)
			|| self.badge.permission().contains(BadgePermission::Admin)
	}

	pub fn has_paint(&self, permission: PaintPermission) -> bool {
		self.is_admin()
			|| self.paint.permission().contains(permission)
			|| self.paint.permission().contains(PaintPermission::Admin)
	}

	pub fn has_user(&self, permission: UserPermission) -> bool {
		self.is_admin()
			|| self.user.permission().contains(permission)
			|| self.user.permission().contains(UserPermission::Admin)
	}

	pub fn has_feature(&self, permission: FeaturePermission) -> bool {
		self.is_admin()
			|| self.feature.permission().contains(permission)
			|| self.feature.permission().contains(FeaturePermission::Admin)
	}

	pub fn has_ticket(&self, permission: TicketPermission) -> bool {
		self.is_admin()
			|| self.ticket.permission().contains(permission)
			|| self.ticket.permission().contains(TicketPermission::Admin)
	}

	pub fn has_admin(&self, permission: AdminPermission) -> bool {
		self.admin.permission().contains(permission) || self.admin.permission().contains(AdminPermission::SuperAdmin)
	}

	pub fn has_emote_set_count_limit(&self, count: u16) -> bool {
		self.has_feature(FeaturePermission::BypassEmoteSetCountLimit)
			|| self.emote_set_count_limit.map_or(true, |limit| count <= limit)
	}

	pub fn has_emote_set_slots_limit(&self, count: u16) -> bool {
		self.has_feature(FeaturePermission::ByPassEmoteSetSlotsLimit)
			|| self.emote_set_slots_limit.map_or(true, |limit| count <= limit)
	}

	pub fn has_personal_emote_set_slots_limit(&self, count: u16) -> bool {
		self.has_feature(FeaturePermission::ByPassPersonalEmoteSetSlotsLimit)
			|| self.personal_emote_set_slots_limit.map_or(true, |limit| count <= limit)
	}

	pub fn is_admin(&self) -> bool {
		self.admin
			.permission()
			.intersects(AdminPermission::Admin | AdminPermission::SuperAdmin)
	}

	pub fn is_super_admin(&self) -> bool {
		self.admin.permission().contains(AdminPermission::SuperAdmin)
	}
}

impl FromIterator<Permissions> for Permissions {
	fn from_iter<I: IntoIterator<Item = Permissions>>(iter: I) -> Self {
		let mut permissions = Self::default();

		for permission in iter {
			permissions.merge(permission);
		}

		permissions
	}
}

impl<'a> FromIterator<&'a Permissions> for Permissions {
	fn from_iter<I: IntoIterator<Item = &'a Permissions>>(iter: I) -> Self {
		let mut permissions = Self::default();

		for permission in iter {
			permissions.merge_ref(permission);
		}

		permissions
	}
}

impl FromIterator<Permission> for Permissions {
	fn from_iter<I: IntoIterator<Item = Permission>>(iter: I) -> Self {
		let mut permissions = Self::default();

		for permission in iter {
			permissions.apply(permission);
		}

		permissions
	}
}

#[derive(Debug, Clone, Copy, EnumImpl)]
pub enum Permission {
	#[enum_impl(impl from)]
	Emote(EmotePermission),
	#[enum_impl(impl from)]
	Role(RolePermission),
	#[enum_impl(impl from)]
	EmoteSet(EmoteSetPermission),
	#[enum_impl(impl from)]
	Badge(BadgePermission),
	#[enum_impl(impl from)]
	Paint(PaintPermission),
	#[enum_impl(impl from)]
	User(UserPermission),
	#[enum_impl(impl from)]
	Feature(FeaturePermission),
	#[enum_impl(impl from)]
	Ticket(TicketPermission),
	#[enum_impl(impl from)]
	Admin(AdminPermission),

	EmoteSetCount(u16),
	EmoteSetSlots(u16),
	PersonalEmoteSetSlots(u16),
}

pub trait PermissionsExt {
	fn has_permission(&self, permission: impl Into<Permission>) -> bool;

	fn has_any_permission(&self, permission: impl IntoIterator<Item = Permission>) -> bool {
		permission.into_iter().any(|permission| self.has_permission(permission))
	}

	fn has_all_permissions(&self, permission: impl IntoIterator<Item = Permission>) -> bool {
		permission.into_iter().all(|permission| self.has_permission(permission))
	}
}

impl PermissionsExt for Permissions {
	fn has_permission(&self, permission: impl Into<Permission>) -> bool {
		match permission.into() {
			Permission::Emote(perm) => self.has_emote(perm),
			Permission::Role(perm) => self.has_role(perm),
			Permission::EmoteSet(perm) => self.has_emote_set(perm),
			Permission::Badge(perm) => self.has_badge(perm),
			Permission::Paint(perm) => self.has_paint(perm),
			Permission::User(perm) => self.has_user(perm),
			Permission::Feature(perm) => self.has_feature(perm),
			Permission::Ticket(perm) => self.has_ticket(perm),
			Permission::Admin(perm) => self.has_admin(perm),

			Permission::EmoteSetCount(perm) => self.has_emote_set_count_limit(perm),
			Permission::EmoteSetSlots(perm) => self.has_emote_set_slots_limit(perm),
			Permission::PersonalEmoteSetSlots(perm) => self.has_personal_emote_set_slots_limit(perm),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_permissions_cmp() {
		let permissions = Permissions {
			emote: AllowDeny {
				allow: EmotePermission::Upload,
				deny: EmotePermission::Delete,
			},
			role: AllowDeny {
				allow: RolePermission::Assign,
				deny: RolePermission::Admin,
			},
			emote_set: AllowDeny {
				allow: EmoteSetPermission::Create,
				deny: EmoteSetPermission::Admin,
			},
			badge: AllowDeny {
				allow: BadgePermission::Admin,
				deny: BadgePermission::none(),
			},
			paint: AllowDeny {
				allow: PaintPermission::Admin,
				deny: PaintPermission::none(),
			},
			user: AllowDeny {
				allow: UserPermission::Ban,
				deny: UserPermission::none(),
			},
			feature: AllowDeny {
				allow: FeaturePermission::UseBadge,
				deny: FeaturePermission::none(),
			},
			ticket: AllowDeny {
				allow: TicketPermission::Read,
				deny: TicketPermission::none(),
			},
			admin: AllowDeny {
				allow: AdminPermission::Admin,
				deny: AdminPermission::none(),
			},
			emote_set_count_limit: Some(10),
			emote_set_slots_limit: Some(5),
			personal_emote_set_slots_limit: Some(3),
			unknown: HashMap::new(),
		};

		assert!(permissions.has_permission(Permission::Emote(EmotePermission::Upload)));
		assert!(permissions.has_permission(Permission::Emote(EmotePermission::Delete)));
	}

	#[test]
	fn test_serialize() {
		let permissions = Permissions {
			emote: AllowDeny {
				allow: EmotePermission::Upload,
				deny: EmotePermission::Delete,
			},
			role: AllowDeny {
				allow: RolePermission::Assign,
				deny: RolePermission::Admin,
			},
			emote_set: AllowDeny {
				allow: EmoteSetPermission::Create,
				deny: EmoteSetPermission::Admin,
			},
			badge: AllowDeny {
				allow: BadgePermission::Admin,
				deny: BadgePermission::none(),
			},
			paint: AllowDeny {
				allow: PaintPermission::Admin,
				deny: PaintPermission::none(),
			},
			user: AllowDeny {
				allow: UserPermission::Ban,
				deny: UserPermission::none(),
			},
			feature: AllowDeny {
				allow: FeaturePermission::UseBadge,
				deny: FeaturePermission::none(),
			},
			ticket: AllowDeny {
				allow: TicketPermission::Read,
				deny: TicketPermission::none(),
			},
			admin: AllowDeny {
				allow: AdminPermission::Admin,
				deny: AdminPermission::none(),
			},
			emote_set_count_limit: Some(10),
			emote_set_slots_limit: Some(5),
			personal_emote_set_slots_limit: Some(3),
			unknown: HashMap::new(),
		};

		let json = serde_json::to_string(&permissions).unwrap();

		assert_eq!(
			json,
			r#"{"emote":{"allow":1,"deny":2},"role":{"allow":1,"deny":2},"emote_set":{"allow":1,"deny":4},"badge":{"allow":1},"paint":{"allow":1},"user":{"allow":1},"feature":{"allow":4},"ticket":{"allow":2},"admin":{"allow":1},"emote_set_count_limit":10,"emote_set_slots_limit":5,"personal_emote_set_slots_limit":3}"#
		);

		let permissions2: Permissions = serde_json::from_str(&json).unwrap();

		assert_eq!(permissions, permissions2, "permissions not equal");

		let permissions = Permissions::default();

		let json = serde_json::to_string(&permissions).unwrap();

		assert_eq!(json, r#"{}"#);

		let permissions = Permissions {
			unknown: vec![("emote2".to_string(), serde_json::json!({"allow": 1, "deny": 2}))]
				.into_iter()
				.collect(),
			..Permissions::default()
		};

		let json = serde_json::to_string(&permissions).unwrap();

		assert_eq!(json, r#"{"emote2":{"allow":1,"deny":2}}"#);
	}

	#[test]
	fn test_deserialize() {
		let json = r#"{}"#;

		let permissions: Permissions = serde_json::from_str(json).unwrap();

		assert_eq!(permissions, Permissions::default(),);

		let json = r#"{"emote2":{"allow":1,"deny":2}}"#;

		let permissions: Permissions = serde_json::from_str(json).unwrap();

		assert_eq!(
			permissions,
			Permissions {
				unknown: vec![("emote2".to_string(), serde_json::json!({"allow": 1, "deny": 2}))]
					.into_iter()
					.collect(),
				..Permissions::default()
			},
		);
	}
}
