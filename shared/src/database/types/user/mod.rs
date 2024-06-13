mod ban;
mod ban_role;
mod connection;
mod editor;
mod presence;
mod relation;
mod session;
mod settings;

use std::sync::Arc;

use hyper::StatusCode;

pub use self::ban::*;
pub use self::ban_role::*;
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
				if self.grants.role_ids.contains(&role.id) {
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
