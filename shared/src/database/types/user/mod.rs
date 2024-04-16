mod ban;
mod connection;
mod editor;
mod gift;
mod presence;
mod product;
mod profile_picture;
mod relation;
mod session;
mod settings;

use std::sync::Arc;

use bson::oid::ObjectId;
use hyper::StatusCode;
use scuffle_utils::http::ext::OptionExt;
use scuffle_utils::http::router::error::RouterError;

pub use self::ban::*;
pub use self::connection::*;
pub use self::editor::*;
pub use self::gift::*;
pub use self::presence::*;
pub use self::product::*;
pub use self::profile_picture::*;
pub use self::relation::*;
pub use self::session::*;
pub use self::settings::*;
use super::{FileSet, ImageFormat};
use crate::database::Collection;
use crate::types::old::{
	CosmeticBadgeModel, CosmeticPaintModel, ImageFormat as ImageFormatOld, ImageHostKind, UserConnectionModel,
	UserConnectionPartialModel, UserModel, UserPartialModel, UserStyle, UserTypeModel,
};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct User {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub email: String,
	pub email_verified: bool,
	pub password_hash: String,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub settings: UserSettings,
	pub two_fa: UserTwoFa,
	pub active_cosmetics: UserActiveCosmetics,
	pub entitled_cache: UserEntitledCache,
	pub active_emote_set_ids: Vec<ObjectId>,
	pub granted_badge_ids: Vec<ObjectId>,
	pub granted_role_ids: Vec<ObjectId>,
	pub granted_paint_ids: Vec<ObjectId>,
	pub granted_emote_set_ids: Vec<ObjectId>,
}

impl Collection for User {
	const NAME: &'static str = "users";
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
pub struct UserActiveCosmetics {
	pub badge_id: Option<ObjectId>,
	pub paint_id: Option<ObjectId>,
	pub profile_picture_id: Option<ObjectId>,
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

impl User {
	pub fn into_old_model_partial(
		self,
		connections: Vec<UserConnection>,
		profile_picture_file_set: &Option<FileSet>,
		paint: Option<CosmeticPaintModel>,
		badge: Option<CosmeticBadgeModel>,
		cdn_base_url: &str,
	) -> Option<UserPartialModel> {
		let main_connection = connections.iter().find(|c| c.main_connection)?;

		let avatar_url = match profile_picture_file_set {
			Some(f) => {
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
			}
			None => None,
		};

		// let paint = match self.active_cosmetics.paint_id {
		// 	Some(id) if self.entitled_cache.paint_ids.contains(&id) => {
		// 		match global.paint_by_id_loader().load(id).await.ok()? {
		// 			Some(p) => Some(p.into_old_model().await.ok()?),
		// 			None => None,
		// 		}
		// 	}
		// 	_ => None,
		// };

		// let badge = match self.active_cosmetics.badge_id {
		// 	Some(id) if self.entitled_cache.badge_ids.contains(&id) => {
		// 		match global.badge_by_id_loader().load(id).await.ok()? {
		// 			Some(b) => Some(b.into_old_model().await.ok()?),
		// 			None => None,
		// 		}
		// 	}
		// 	_ => None,
		// };

		Some(UserPartialModel {
			id: self.id,
			user_type: UserTypeModel::Regular,
			username: main_connection.platform_username.clone(),
			display_name: main_connection.platform_display_name.clone(),
			avatar_url: avatar_url.unwrap_or_default(),
			style: UserStyle {
				color: 0,
				paint_id: self.active_cosmetics.paint_id,
				paint,
				badge_id: self.active_cosmetics.badge_id,
				badge,
			},
			role_ids: self.entitled_cache.role_ids.into_iter().collect(),
			connections: connections.into_iter().map(UserConnectionPartialModel::from).collect(),
		})
	}

	pub fn into_old_model(
		self,
		connections: Vec<UserConnection>,
		profile_picture_file_set: &Option<FileSet>,
		paint: Option<CosmeticPaintModel>,
		badge: Option<CosmeticBadgeModel>,
		cdn_base_url: &str,
	) -> Option<UserModel> {
		let created_at = self.id.timestamp().timestamp_millis();
		let partial = self.into_old_model_partial(connections, profile_picture_file_set, paint, badge, cdn_base_url)?;

		Some(UserModel {
			id: partial.id,
			user_type: partial.user_type,
			username: partial.username,
			display_name: partial.display_name,
			created_at: created_at as i64,
			avatar_url: partial.avatar_url,
			biography: String::new(),
			style: partial.style,
			emote_sets: todo!(),
			editors: todo!(),
			role_ids: partial.role_ids,
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
					emote_set: todo!(),
					user: None,
				})
				.collect(),
		})
	}
}
