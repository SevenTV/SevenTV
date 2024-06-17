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

#[bitmask(i32)]
pub enum EmotePermission {
	/// Grants all permissions
	Admin = 1,
	/// Allows the user to upload emotes
	Upload = 2,
	/// Allows the user to delete their own emotes
	/// Note: if the emote has more than a certain number of uses, the emote
	/// cannot be deleted unless the user has the `Admin` permission
	Delete = 4,
	/// Allows the user to edit their own emotes
	Edit = 8,
	/// Allows the user to manage emotes (edit, delete) any emote
	/// Note: this permission does not allow the user to delete emotes with more
	/// than a certain number of uses
	ManageAny = 16,
	/// Allows to merge emotes together
	Merge = 32,
	/// Allows the user to view unlisted emotes
	ViewUnlisted = 64,
}

impl Default for EmotePermission {
	fn default() -> Self {
		Self::none()
	}
}

impl BitMask for EmotePermission {
	type Bits = i32;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

#[bitmask(i32)]
pub enum RolePermission {
	/// Grants all permissions
	Admin = 1,
	/// Allows the user to create roles
	Manage = 2,
	/// Allows the user to assign roles to objects
	Assign = 4,
}

impl BitMask for RolePermission {
	type Bits = i32;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for RolePermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(i32)]
pub enum EmoteSetPermission {
	/// Grants all permissions
	Admin = 1,
	/// Allows the user to manage their own emote sets
	Manage = 2,
	/// Allows the user to manage any user owned emote set
	ManageAny = 4,
	/// Allows the user to resize the capacity of an emote set (they can manage)
	Resize = 8,
	/// Allows the user to manage any global emote set
	ManageGlobal = 16,
	/// Allows the user to manage special emote sets
	ManageSpecial = 32,
	/// Allows the user to assign emote sets to objects
	Assign = 64,
}

impl BitMask for EmoteSetPermission {
	type Bits = i32;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for EmoteSetPermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(i32)]
pub enum BadgePermission {
	/// Grants all permissions
	Admin = 1,
	/// Allows the user to manage badges
	Manage = 2,
	/// Assign badges to objects
	Assign = 4,
}

impl BitMask for BadgePermission {
	type Bits = i32;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for BadgePermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(i32)]
pub enum PaintPermission {
	/// Grants all permissions
	Admin = 1,
	/// Manage paints
	Manage = 2,
	/// Allows the user to assign paints to objects
	Assign = 4,
}

impl BitMask for PaintPermission {
	type Bits = i32;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for PaintPermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(i32)]
pub enum FlagPermission {
	/// Hidden from search results
	Hidden = 1,
}

impl BitMask for FlagPermission {
	type Bits = i32;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for FlagPermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(i32)]
pub enum UserPermission {
	/// Grants all permissions
	Admin = 1,
	/// Allows the user to login to the site
	Login = 2,
	/// Allows the user to invite editors
	InviteEditors = 4,

	/// Allows the user to upload a custom profile picture, and use it.
	/// Note: Without this permission uploaded profile pictures will not be
	/// returned in the API People who can manage users can upload / change the
	/// profile picture of other users but they may not be able to use it
	UseCustomProfilePicture = 8,
	/// Use personal emote sets
	UsePersonalEmoteSet = 8,
	/// Use badges
	UseBadge = 16,
	/// Use paints
	UsePaint = 32,
	/// Allows the user to use special emote sets
	UseSpecialEmoteSet = 64,

	/// Allows the user to manage other users
	ManageAny = 128,

	/// Allows the user to moderate other users (ban, unban, etc.)
	Moderate = 256,

	/// View hidden users
	ViewHidden = 512,
}

impl BitMask for UserPermission {
	type Bits = i32;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for UserPermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(i32)]
pub enum TicketPermission {
	/// Grants all permissions
	Admin = 1,
	/// Allows the user to create tickets
	Create = 2,
	/// Allows the user to manage tickets related to abuse
	ManageAbuse = 4,
	/// Allows the user to manage billing tickets
	ManageBilling = 8,
	/// Allows the user to manage generic tickets
	ManageGeneric = 16,
	/// Allows messages to be sent to the ticket
	Message = 32,
}

impl BitMask for TicketPermission {
	type Bits = i32;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for TicketPermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(i32)]
pub enum EmoteModerationRequestPermission {
	/// Grants all permissions
	Admin = 1,
	/// Allows the user to manage emote moderation requests
	Manage = 2,
}

impl BitMask for EmoteModerationRequestPermission {
	type Bits = i32;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for EmoteModerationRequestPermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(i32)]
pub enum ReportPermission {
	/// Grants all permissions
	Admin = 1,
	/// Allows the user to create reports
	Create = 2,
	/// Allows the user to manage reports
	Manage = 4,
}

impl BitMask for ReportPermission {
	type Bits = i32;

	fn bits(&self) -> Self::Bits {
		self.bits()
	}
}

impl Default for ReportPermission {
	fn default() -> Self {
		Self::none()
	}
}

#[bitmask(i32)]
pub enum AdminPermission {
	/// Grants all permissions
	Admin = 1,
	/// Grants all permissions and ignores role hierarchy
	SuperAdmin = 2,
}

impl BitMask for AdminPermission {
	type Bits = i32;

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
	pub ticket: AllowDeny<TicketPermission>,
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub emote_moderation_request: AllowDeny<EmoteModerationRequestPermission>,
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub report: AllowDeny<ReportPermission>,
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub admin: AllowDeny<AdminPermission>,
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub flags: AllowDeny<FlagPermission>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub emote_moderation_request_priority: Option<i32>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub emote_moderation_request_limit: Option<i32>,

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
		self.ticket.merge(other.ticket);
		self.admin.merge(other.admin);
		self.emote_moderation_request_priority = other
			.emote_moderation_request_priority
			.or(self.emote_moderation_request_priority);
		self.emote_moderation_request_limit = other.emote_moderation_request_limit.or(self.emote_moderation_request_limit);
	}

	pub fn allow(&mut self, perm: impl Into<Permission>) {
		match perm.into() {
			Permission::Emote(perm) => self.emote.allow(perm),
			Permission::Role(perm) => self.role.allow(perm),
			Permission::EmoteSet(perm) => self.emote_set.allow(perm),
			Permission::Badge(perm) => self.badge.allow(perm),
			Permission::Paint(perm) => self.paint.allow(perm),
			Permission::User(perm) => self.user.allow(perm),
			Permission::Ticket(perm) => self.ticket.allow(perm),
			Permission::EmoteModerationRequest(perm) => self.emote_moderation_request.allow(perm),
			Permission::Report(perm) => self.report.allow(perm),
			Permission::Admin(perm) => self.admin.allow(perm),
			Permission::Flags(perm) => self.flags.allow(perm),
		}
	}

	pub fn deny(&mut self, perm: impl Into<Permission>) {
		match perm.into() {
			Permission::Emote(perm) => self.emote.deny(perm),
			Permission::Role(perm) => self.role.deny(perm),
			Permission::EmoteSet(perm) => self.emote_set.deny(perm),
			Permission::Badge(perm) => self.badge.deny(perm),
			Permission::Paint(perm) => self.paint.deny(perm),
			Permission::User(perm) => self.user.deny(perm),
			Permission::Ticket(perm) => self.ticket.deny(perm),
			Permission::EmoteModerationRequest(perm) => self.emote_moderation_request.deny(perm),
			Permission::Report(perm) => self.report.deny(perm),
			Permission::Admin(perm) => self.admin.deny(perm),
			Permission::Flags(perm) => self.flags.deny(perm),
		}
	}

	pub fn denied(&self, permission: impl Into<Permission>) -> bool {
		self.denied_permission(permission)
	}

	pub fn denied_emote(&self, permission: EmotePermission) -> bool {
		!self.is_admin() && !self.emote.permission().contains(EmotePermission::Admin) && self.emote.deny.contains(permission)
	}

	pub fn denied_role(&self, permission: RolePermission) -> bool {
		!self.is_admin() && !self.role.permission().contains(RolePermission::Admin) && self.role.deny.contains(permission)
	}

	pub fn denied_emote_set(&self, permission: EmoteSetPermission) -> bool {
		!self.is_admin()
			&& !self.emote_set.permission().contains(EmoteSetPermission::Admin)
			&& self.emote_set.deny.contains(permission)
	}

	pub fn denied_badge(&self, permission: BadgePermission) -> bool {
		!self.is_admin() && !self.badge.permission().contains(BadgePermission::Admin) && self.badge.deny.contains(permission)
	}

	pub fn denied_paint(&self, permission: PaintPermission) -> bool {
		!self.is_admin() && !self.paint.permission().contains(PaintPermission::Admin) && self.paint.deny.contains(permission)
	}

	pub fn denied_user(&self, permission: UserPermission) -> bool {
		!self.is_admin() && !self.user.permission().contains(UserPermission::Admin) && self.user.deny.contains(permission)
	}

	pub fn denied_ticket(&self, permission: TicketPermission) -> bool {
		!self.is_admin()
			&& !self.ticket.permission().contains(TicketPermission::Admin)
			&& self.ticket.deny.contains(permission)
	}

	pub fn denied_emote_moderation_request(&self, permission: EmoteModerationRequestPermission) -> bool {
		!self.is_admin()
			&& !self.emote_moderation_request.permission().contains(EmoteModerationRequestPermission::Admin)
			&& self.emote_moderation_request.deny.contains(permission)
	}

	pub fn denied_report(&self, permission: ReportPermission) -> bool {
		!self.is_admin()
			&& !self.report.permission().contains(ReportPermission::Admin)
			&& self.report.deny.contains(permission)
	}

	pub fn denied_admin(&self, permission: AdminPermission) -> bool {
		!self.admin.permission().contains(AdminPermission::SuperAdmin) && self.admin.deny.contains(permission)
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

	pub fn has_ticket(&self, permission: TicketPermission) -> bool {
		self.is_admin()
			|| self.ticket.permission().contains(permission)
			|| self.ticket.permission().contains(TicketPermission::Admin)
	}

	pub fn has_emote_moderation_request(&self, permission: EmoteModerationRequestPermission) -> bool {
		self.is_admin()
			|| self.emote_moderation_request.permission().contains(permission)
			|| self.emote_moderation_request.permission().contains(EmoteModerationRequestPermission::Admin)
	}

	pub fn has_report(&self, permission: ReportPermission) -> bool {
		self.is_admin()
			|| self.report.permission().contains(permission)
			|| self.report.permission().contains(ReportPermission::Admin)
	}

	pub fn has_admin(&self, permission: AdminPermission) -> bool {
		self.admin.permission().contains(permission) || self.admin.permission().contains(AdminPermission::SuperAdmin)
	}

	pub fn is_admin(&self) -> bool {
		self.admin
			.permission()
			.intersects(AdminPermission::Admin | AdminPermission::SuperAdmin)
	}

	pub fn is_super_admin(&self) -> bool {
		self.admin.permission().contains(AdminPermission::SuperAdmin)
	}

	pub fn has_flags(&self, permission: FlagPermission) -> bool {
		self.flags.permission().contains(permission)
	}

	pub fn denied_flags(&self, permission: FlagPermission) -> bool {
		self.flags.deny.contains(permission)
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
			permissions.allow(permission);
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
	Ticket(TicketPermission),
	#[enum_impl(impl from)]
	EmoteModerationRequest(EmoteModerationRequestPermission),
	#[enum_impl(impl from)]
	Report(ReportPermission),
	#[enum_impl(impl from)]
	Admin(AdminPermission),
	#[enum_impl(impl from)]
	Flags(FlagPermission),
}

pub trait PermissionsExt {
	fn has_permission(&self, permission: impl Into<Permission>) -> bool;

	fn denied_permission(&self, permission: impl Into<Permission>) -> bool;

	fn has_any_permission(&self, permission: impl IntoIterator<Item = Permission>) -> bool {
		permission.into_iter().any(|permission| self.has_permission(permission))
	}

	fn has_all_permissions(&self, permission: impl IntoIterator<Item = Permission>) -> bool {
		permission.into_iter().all(|permission| self.has_permission(permission))
	}

	fn denied_any_permission(&self, permission: impl IntoIterator<Item = Permission>) -> bool {
		permission.into_iter().any(|permission| self.denied_permission(permission))
	}

	fn denied_all_permissions(&self, permission: impl IntoIterator<Item = Permission>) -> bool {
		permission.into_iter().all(|permission| self.denied_permission(permission))
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
			Permission::Ticket(perm) => self.has_ticket(perm),
			Permission::EmoteModerationRequest(perm) => self.has_emote_moderation_request(perm),
			Permission::Report(perm) => self.has_report(perm),
			Permission::Admin(perm) => self.has_admin(perm),
			Permission::Flags(perm) => self.has_flags(perm),
		}
	}

	fn denied_permission(&self, permission: impl Into<Permission>) -> bool {
		match permission.into() {
			Permission::Emote(perm) => self.denied_emote(perm),
			Permission::Role(perm) => self.denied_role(perm),
			Permission::EmoteSet(perm) => self.denied_emote_set(perm),
			Permission::Badge(perm) => self.denied_badge(perm),
			Permission::Paint(perm) => self.denied_paint(perm),
			Permission::User(perm) => self.denied_user(perm),
			Permission::Ticket(perm) => self.denied_ticket(perm),
			Permission::EmoteModerationRequest(perm) => self.denied_emote_moderation_request(perm),
			Permission::Report(perm) => self.denied_report(perm),
			Permission::Admin(perm) => self.denied_admin(perm),
			Permission::Flags(perm) => self.denied_flags(perm),
		}
	}
}
