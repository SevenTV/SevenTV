mod ban;
mod connection;
mod editor;
mod presence;
mod relation;
mod session;
mod settings;

use std::sync::Arc;

use hyper::StatusCode;

pub use self::ban::*;
pub use self::connection::*;
pub use self::editor::*;
pub use self::presence::*;
pub use self::relation::*;
pub use self::session::*;
pub use self::settings::*;
use super::{BadgeId, EmoteSetId, ImageSet, PaintId, Permissions, ProductId, Role, RoleId};
use crate::database::{Collection, Id};

pub type UserId = Id<User>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct User {
	#[serde(rename = "_id")]
	pub id: UserId,
	pub email: Option<String>,
	pub email_verified: bool,
	pub password_hash: Option<String>,
	pub settings: UserSettings,
	pub two_fa: Option<UserTwoFa>,
	pub style: UserStyle,
	pub active_emote_set_ids: Vec<EmoteSetId>,
	pub grants: UserGrants,
	pub entitled_cache: UserEntitledCache,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserGrants {
	pub role_ids: Vec<RoleId>,
	pub badge_ids: Vec<BadgeId>,
	pub paint_ids: Vec<PaintId>,
	pub emote_set_ids: Vec<EmoteSetId>,
}

impl User {
	pub fn compute_permissions(&self, roles: &[Role]) -> Permissions {
		roles
			.iter()
			.filter_map(|role| {
				if self.entitled_cache.role_ids.contains(&role.id) {
					Some(&role.permissions)
				} else {
					None
				}
			})
			.collect()
	}
}

impl Collection for User {
	const COLLECTION_NAME: &'static str = "users";
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
	pub active_profile_picture: Option<ImageSet>,
	pub all_profile_pictures: Vec<ImageSet>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserEntitledCache {
	pub role_ids: Vec<RoleId>,
	pub badge_ids: Vec<BadgeId>,
	pub emote_set_ids: Vec<EmoteSetId>,
	pub paint_ids: Vec<PaintId>,
	pub product_ids: Vec<ProductId>,
	pub invalidated_at: mongodb::bson::DateTime,
}

impl Default for UserEntitledCache {
	fn default() -> Self {
		Self {
			role_ids: Default::default(),
			badge_ids: Default::default(),
			emote_set_ids: Default::default(),
			paint_ids: Default::default(),
			product_ids: Default::default(),
			invalidated_at: mongodb::bson::DateTime::now(),
		}
	}
}

impl UserEntitledCache {
	pub fn dedup(&mut self) {
		self.role_ids.sort_unstable();
		self.role_ids.dedup();

		self.badge_ids.sort_unstable();
		self.badge_ids.dedup();

		self.emote_set_ids.sort_unstable();
		self.emote_set_ids.dedup();

		self.paint_ids.sort_unstable();
		self.paint_ids.dedup();

		self.product_ids.sort_unstable();
		self.product_ids.dedup();
	}
}
