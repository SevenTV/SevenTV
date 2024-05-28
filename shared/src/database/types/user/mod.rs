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

use super::ImageSet;
use super::ProductId;
use super::{BadgeId, EmoteSetId, PaintId, Permissions, Role, RoleId};
use crate::database::{Collection, Id};
use crate::types::old::{
	CosmeticBadgeModel, CosmeticPaintModel, EmoteSetPartialModel, ImageFormat as ImageFormatOld, ImageHostKind,
	UserConnectionModel, UserConnectionPartialModel, UserEditorModel, UserModel, UserPartialModel,
	UserStyle as UserStyleOld, UserTypeModel,
};

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

impl User {
	pub fn into_old_model_partial(
		self,
		connections: Vec<UserConnection>,
		paint: Option<CosmeticPaintModel>,
		badge: Option<CosmeticBadgeModel>,
		cdn_base_url: &str,
	) -> UserPartialModel {
		let main_connection = connections.iter().find(|c| c.main_connection);

		let avatar_url = self
			.style
			.active_profile_picture
			.and_then(|s| s.outputs.iter().max_by_key(|i| i.size).map(|i| i.get_url(cdn_base_url)))
			.or(main_connection.and_then(|c| c.platform_avatar_url.clone()));

		UserPartialModel {
			id: self.id,
			user_type: UserTypeModel::Regular,
			username: main_connection.map(|c| c.platform_username.clone()).unwrap_or_default(),
			display_name: main_connection.map(|c| c.platform_display_name.clone()).unwrap_or_default(),
			avatar_url: avatar_url.unwrap_or_default(),
			style: UserStyleOld {
				color: 0,
				paint_id: self.style.active_paint_id,
				paint,
				badge_id: self.style.active_badge_id,
				badge,
			},
			role_ids: self.entitled_cache.role_ids.into_iter().collect(),
			connections: connections.into_iter().map(UserConnectionPartialModel::from).collect(),
		}
	}

	pub fn into_old_model(
		self,
		connections: Vec<UserConnection>,
		paint: Option<CosmeticPaintModel>,
		badge: Option<CosmeticBadgeModel>,
		emote_sets: Vec<EmoteSetPartialModel>,
		editors: Vec<UserEditorModel>,
		cdn_base_url: &str,
	) -> UserModel {
		let created_at = self.id.timestamp_ms();
		let partial = self.into_old_model_partial(connections, paint, badge, cdn_base_url);

		UserModel {
			id: partial.id,
			user_type: partial.user_type,
			username: partial.username,
			display_name: partial.display_name,
			created_at,
			avatar_url: partial.avatar_url,
			biography: String::new(),
			style: partial.style,
			emote_sets,
			editors,
			roles: partial.role_ids,
			connections: partial
				.connections
				.into_iter()
				.map(|p| UserConnectionModel {
					id: p.id,
					platform: p.platform,
					username: p.username,
					display_name: p.display_name,
					linked_at: p.linked_at,
					emote_capacity: p.emote_capacity,
					emote_set_id: p.emote_set_id,
					emote_set: None,
					user: None,
				})
				.collect(),
		}
	}
}
