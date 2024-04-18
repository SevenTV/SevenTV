mod ban;
mod connection;
mod editor;
mod gift;
mod presence;
mod product;
mod relation;
mod session;
mod settings;

use std::sync::Arc;

use bson::oid::ObjectId;
use bson::Bson;
use hyper::StatusCode;
use scuffle_utils::http::ext::OptionExt;
use scuffle_utils::http::router::error::RouterError;

pub use self::ban::*;
pub use self::connection::*;
pub use self::editor::*;
pub use self::gift::*;
pub use self::presence::*;
pub use self::product::*;
pub use self::relation::*;
pub use self::session::*;
pub use self::settings::*;
use super::{FileSet, ImageFormat, Permissions, Role};
use crate::database::Collection;
use crate::types::old::{
	CosmeticBadgeModel, CosmeticPaintModel, EmoteSetPartialModel, ImageFormat as ImageFormatOld, ImageHostKind,
	UserConnectionModel, UserConnectionPartialModel, UserEditorModel, UserModel, UserPartialModel,
	UserStyle as UserStyleOld, UserTypeModel,
};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct User {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub email: Option<String>,
	pub email_verified: bool,
	pub password_hash: Option<String>,
	pub settings: UserSettings,
	pub two_fa: Option<UserTwoFa>,
	pub style: UserStyle,
	pub active_emote_set_ids: Vec<ObjectId>,
	pub grants: UserGrants,
	pub entitled_cache: UserEntitledCache,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserGrants {
	pub role_ids: Vec<ObjectId>,
	pub badge_ids: Vec<ObjectId>,
	pub paint_ids: Vec<ObjectId>,
	pub emote_set_ids: Vec<ObjectId>,
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
	pub active_badge_id: Option<ObjectId>,
	pub active_paint_id: Option<ObjectId>,
	pub active_profile_picture_id: Option<ObjectId>,
	pub all_profile_picture_ids: Vec<ObjectId>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserEntitledCache {
	pub role_ids: Vec<ObjectId>,
	pub badge_ids: Vec<ObjectId>,
	pub emote_set_ids: Vec<ObjectId>,
	pub paint_ids: Vec<ObjectId>,
	pub product_ids: Vec<ObjectId>,
	pub invalidated_at: chrono::DateTime<chrono::Utc>,
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
		profile_picture_file_set: Option<&FileSet>,
		paint: Option<CosmeticPaintModel>,
		badge: Option<CosmeticBadgeModel>,
		cdn_base_url: &str,
	) -> UserPartialModel {
		let main_connection = connections.iter().find(|c| c.main_connection);

		let avatar_url = profile_picture_file_set.and_then(|f| {
			let file = f.properties.default_image()?;

			Some(
				ImageHostKind::ProfilePicture.create_full_url(
					cdn_base_url,
					f.id,
					file.extra.scale,
					file.extra
						.variants
						.iter()
						.find(|v| v.format == ImageFormat::Webp)
						.map(|_| ImageFormatOld::Webp)?,
				),
			)
		});

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
		profile_picture_file_set: Option<&FileSet>,
		paint: Option<CosmeticPaintModel>,
		badge: Option<CosmeticBadgeModel>,
		emote_sets: Vec<EmoteSetPartialModel>,
		editors: Vec<UserEditorModel>,
		cdn_base_url: &str,
	) -> UserModel {
		let created_at = self.id.timestamp().timestamp_millis();
		let partial = self.into_old_model_partial(connections, profile_picture_file_set, paint, badge, cdn_base_url);

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