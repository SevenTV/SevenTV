use serde::Deserialize;

use super::UserId;
use crate::database::role::permissions::{Permission, Permissions, PermissionsExt};
use crate::database::{Id, MongoCollection};

pub type UserBanId = Id<UserBan>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection, PartialEq, Eq)]
#[mongo(collection_name = "user_bans")]
#[mongo(index(fields(user_id = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(search = "crate::typesense::types::user::ban::UserBan")]
pub struct UserBan {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: UserBanId,
	pub user_id: UserId,
	pub created_by_id: UserId,
	pub reason: String,
	pub tags: Vec<String>,
	#[serde(with = "crate::database::serde")]
	pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
	pub removed: Option<UserBanRemoved>,
	pub permissions: Permissions,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UserBanRemoved {
	#[serde(with = "crate::database::serde")]
	pub removed_at: chrono::DateTime<chrono::Utc>,
	pub removed_by_id: UserId,
}

pub struct ActiveBans<'a>(Vec<&'a UserBan>);

impl<'a> ActiveBans<'a> {
	pub fn new(bans: &'a [UserBan]) -> Option<Self> {
		let mut bans = bans
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
		self.0.iter().find(|ban| ban.permissions.denied(permission)).copied()
	}
}
