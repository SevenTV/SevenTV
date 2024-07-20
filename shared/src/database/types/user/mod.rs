pub mod ban;
pub mod ban_template;
pub mod connection;
pub mod editor;
pub mod presence;
pub mod profile_picture;
pub mod relation;
pub mod session;
pub mod settings;

use connection::UserConnection;
use profile_picture::UserProfilePictureId;
use settings::UserSettings;

use super::badge::BadgeId;
use super::emote::EmoteId;
use super::emote_set::EmoteSetId;
use super::entitlement::{CalculatedEntitlements, EntitlementEdge, EntitlementEdgeKind};
use super::paint::PaintId;
use super::role::permissions::{Permission, Permissions, PermissionsExt};
use super::role::RoleId;
use super::MongoGenericCollection;
use crate::database::{Id, MongoCollection};

pub type UserId = Id<User>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "users")]
#[mongo(index(fields("connections.platform" = 1, "connections.platform_id" = 1), unique, sparse))]
#[mongo(index(fields("connections.platform" = 1, "connections.platform_username" = 1)))]
#[mongo(index(fields("cached_active_emotes" = 1)))]
#[mongo(index(fields("cached_entitlements" = 1)))]
#[mongo(index(fields("style.active_emote_set_id" = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct User {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: UserId,
	pub email: Option<String>,
	pub email_verified: bool,
	pub has_bans: bool,
	pub settings: UserSettings,
	pub two_fa: Option<UserTwoFa>,
	pub style: UserStyle,
	pub connections: Vec<UserConnection>,
	pub cached_role_rank: i32,
	#[serde(default)]
	pub cached_entitlements: Vec<EntitlementEdgeKind>,
	#[serde(default)]
	pub cached_active_emotes: Vec<EmoteId>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserTwoFa {
	pub flags: i32,
	pub secret: Vec<u8>,
	pub recovery_codes: Vec<i32>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserStyle {
	pub active_badge_id: Option<BadgeId>,
	pub active_paint_id: Option<PaintId>,
	pub active_emote_set_id: Option<EmoteSetId>,
	pub active_profile_picture: Option<UserProfilePictureId>,
	pub pending_profile_picture: Option<UserProfilePictureId>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	std::iter::once(MongoGenericCollection::new::<User>())
		.chain(ban_template::collections())
		.chain(editor::collections())
		.chain(presence::collections())
		.chain(relation::collections())
		.chain(session::collections())
		.chain(profile_picture::collections())
}

#[derive(Debug, Clone, Default)]
pub struct FullUser {
	pub user: User,
	pub computed: UserComputed,
}

#[derive(Debug, Clone)]
pub enum FullUserRef<'a> {
	Owned(FullUser),
	Ref(&'a FullUser),
}

impl AsRef<FullUser> for FullUserRef<'_> {
	fn as_ref(&self) -> &FullUser {
		match self {
			Self::Owned(user) => user,
			Self::Ref(user) => user,
		}
	}
}

impl std::ops::Deref for FullUserRef<'_> {
	type Target = FullUser;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Owned(user) => user,
			Self::Ref(user) => user,
		}
	}
}

impl PermissionsExt for FullUser {
	fn has(&self, permission: impl Into<Permission>) -> bool {
		self.computed.permissions.has(permission)
	}

	fn denied(&self, permission: impl Into<Permission>) -> bool {
		self.computed.permissions.denied(permission)
	}
}

impl std::ops::Deref for FullUser {
	type Target = User;

	fn deref(&self) -> &Self::Target {
		&self.user
	}
}

impl AsRef<User> for FullUser {
	fn as_ref(&self) -> &User {
		&self.user
	}
}

impl AsRef<UserComputed> for FullUser {
	fn as_ref(&self) -> &UserComputed {
		&self.computed
	}
}

#[derive(Debug, Clone, Default)]
pub struct UserComputed {
	pub permissions: Permissions,
	pub entitlements: CalculatedEntitlements,
	pub highest_role_rank: i32,
	pub highest_role_color: Option<i32>,
	pub roles: Vec<RoleId>,
	pub raw_entitlements: Option<Vec<EntitlementEdge>>,
}
