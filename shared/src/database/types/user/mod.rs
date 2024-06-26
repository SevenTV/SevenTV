pub mod ban;
pub mod ban_template;
pub mod connection;
pub mod editor;
pub mod presence;
pub mod relation;
pub mod session;
pub mod settings;

use ban::UserBan;
use connection::UserConnection;
use settings::UserSettings;

use super::badge::BadgeId;
use super::emote::EmoteId;
use super::emote_set::EmoteSetId;
use super::entitlement::{CalculatedEntitlements, EntitlementEdge, EntitlementGroupId};
use super::image_set::ImageSet;
use super::paint::PaintId;
use super::product::promotion::PromotionId;
use super::product::subscription_timeline::{
	SubscriptionTimelineId, SubscriptionTimelinePeriodId, UserSubscriptionTimelineId,
};
use super::product::ProductId;
use super::role::permissions::{Permission, Permissions, PermissionsExt};
use super::role::RoleId;
use super::GenericCollection;
use crate::database::{Collection, Id};

pub type UserId = Id<User>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct User {
	#[serde(rename = "_id")]
	pub id: UserId,
	pub email: Option<String>,
	pub email_verified: bool,
	pub settings: UserSettings,
	pub two_fa: Option<UserTwoFa>,
	pub style: UserStyle,
	pub connections: Vec<UserConnection>,
	#[serde(default)]
	pub search_index: UserSearchIndex,
	pub bans: Vec<UserBan>,
}

pub struct ActiveBans<'a>(Vec<&'a UserBan>);

impl ActiveBans<'_> {
	pub fn iter(&self) -> impl Iterator<Item = &UserBan> + '_ {
		self.0.iter().copied()
	}

	pub fn permissions(&self) -> Permissions {
		self.0.iter().fold(Permissions::default(), |mut perms, ban| {
			perms.merge_ref(&ban.permissions);
			perms
		})
	}

	pub fn greatest_ban_for(&self, permission: Permission) -> Option<&UserBan> {
		self.0
			.iter()
			.find(|ban| ban.permissions.denied(permission))
			.copied()
	}
}

impl User {
	pub fn active_bans(&self) -> Option<ActiveBans<'_>> {
		let mut bans = self
			.bans
			.iter()
			.filter(|ban| {
				ban.removed.is_none() && (ban.expires_at.is_none() || ban.expires_at.unwrap() > chrono::Utc::now())
			})
			.collect::<Vec<_>>();

		if bans.is_empty() {
			return None;
		}

		// sort bans that do not expire first, followed by bans that expire last

		bans.sort_by(|a, b| match (a.expires_at, b.expires_at) {
			(None, None) => std::cmp::Ordering::Equal,
			(None, Some(_)) => std::cmp::Ordering::Less,
			(Some(_), None) => std::cmp::Ordering::Greater,
			(Some(a), Some(b)) => b.cmp(&a),
		});

		Some(ActiveBans(bans))
	}
}

impl Collection for User {
	const COLLECTION_NAME: &'static str = "users";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"style.active_emote_set_id": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"search_index.self_dirty": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"search_index.emotes_dirty": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"search_index.entitlements_dirty": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"search_index.emote_ids": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"search_index.entitlement_cache_keys": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"connections.platform": 1,
					"connections.platform_id": 1,
				})
				.options(mongodb::options::IndexOptions::builder().unique(true).build())
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"bans.id": 1,
				})
				.build(),
		]
	}
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
	pub active_profile_picture: Option<ImageSet>,
	pub all_profile_pictures: Vec<ImageSet>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct UserSearchIndex {
	/// Each role has a rank, this is used for a search ordering
	pub role_rank: i32,
	/// If some property on the user has changed and we need to update the
	/// search index
	pub self_dirty: Option<Id<()>>,
	/// If specifically the user's emotes have changed
	pub emotes_dirty: Option<Id<()>>,
	/// If specifically the user's entitlements have changed
	pub entitlements_dirty: Option<Id<()>>,
	/// The emote ids that the user has active
	pub emote_ids: Vec<EmoteId>,
	/// A set of keys that we use to determine if entitlements have changed
	pub entitlement_cache_keys: Vec<EntitlementCacheKey>,
	/// A cache of what badges the user is entitled to
	pub entitled_badges: Vec<BadgeId>,
	/// A cache of what paints the user is entitled to
	pub entitled_paints: Vec<PaintId>,
	/// A cache of what emote sets the user is entitled to
	pub entitled_emote_set_ids: Vec<EmoteSetId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum EntitlementCacheKey {
	Role {
		role_id: RoleId,
	},
	Product {
		product_id: ProductId,
	},
	Promotion {
		promotion_id: PromotionId,
	},
	SubscriptionTimeline {
		subscription_timeline_id: SubscriptionTimelineId,
	},
	SubscriptionTimelinePeriod {
		subscription_timeline_id: SubscriptionTimelineId,
		period_id: SubscriptionTimelinePeriodId,
	},
	UserSubscriptionTimeline {
		user_subscription_timeline_id: UserSubscriptionTimelineId,
	},
	EntitlementGroup {
		entitlement_group_id: EntitlementGroupId,
	},
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	std::iter::once(GenericCollection::new::<User>())
		.chain(ban_template::collections())
		.chain(editor::collections())
		.chain(presence::collections())
		.chain(relation::collections())
		.chain(session::collections())
}

#[derive(Debug, Clone)]
pub struct FullUser {
	pub user: User,
	pub computed: UserComputed,
}

#[derive(Debug, Clone)]
pub enum FullUserRef<'a> {
	Owned(FullUser),
	Ref(&'a FullUser),
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

#[derive(Debug, Clone)]
pub struct UserComputed {
	pub permissions: Permissions,
	pub entitlements: CalculatedEntitlements,
	pub highest_role_rank: i32,
	pub highest_role_color: Option<u32>,
	pub raw_entitlements: Option<Vec<EntitlementEdge>>,
}
