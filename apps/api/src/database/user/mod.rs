mod active_emote_set;
mod badge;
mod ban;
mod connection;
mod editor;
mod emote_set;
mod gift;
mod paint;
mod product;
mod profile_picture;
mod relation;
mod roles;
mod session;
mod settings;

use std::sync::Arc;

pub use active_emote_set::*;
pub use badge::*;
pub use ban::*;
pub use connection::*;
pub use editor::*;
pub use emote_set::*;
pub use gift::*;
pub use paint::*;
pub use product::*;
pub use profile_picture::*;
pub use relation::*;
pub use roles::*;
pub use session::*;
pub use settings::*;
use shared::types::old::{UserConnectionPartial, UserModelPartial, UserStyle};

use crate::database::Table;
use crate::global::Global;
use crate::http::v3;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct User {
	pub id: ulid::Ulid,
	pub email: String,
	pub email_verified: bool,
	pub password_hash: String,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub settings: UserSettings,
	#[from_row(flatten)]
	pub two_fa: UserTwoFa,
	#[from_row(flatten)]
	pub active_cosmetics: UserActiveCosmetics,
	#[from_row(flatten)]
	pub entitled_cache: UserEntitledCache,
}

impl Table for User {
	const TABLE_NAME: &'static str = "users";
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserTwoFa {
	#[from_row(rename = "two_fa_flags")]
	pub flags: i32,
	#[from_row(rename = "two_fa_secret")]
	pub secret: Vec<u8>,
	#[from_row(rename = "two_fa_recovery_codes")]
	pub recovery_codes: Vec<i32>,
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserActiveCosmetics {
	#[from_row(rename = "active_badge_id")]
	pub badge_id: Option<ulid::Ulid>,
	#[from_row(rename = "active_paint_id")]
	pub paint_id: Option<ulid::Ulid>,
	#[from_row(rename = "active_profile_picture_id")]
	pub profile_picture_id: Option<ulid::Ulid>,
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserEntitledCache {
	#[from_row(rename = "entitled_cache_role_ids")]
	pub role_ids: Vec<ulid::Ulid>,
	#[from_row(rename = "entitled_cache_badge_ids")]
	pub badge_ids: Vec<ulid::Ulid>,
	#[from_row(rename = "entitled_cache_emote_set_ids")]
	pub emote_set_ids: Vec<ulid::Ulid>,
	#[from_row(rename = "entitled_cache_paint_ids")]
	pub paint_ids: Vec<ulid::Ulid>,
	#[from_row(rename = "entitled_cache_product_ids")]
	pub product_ids: Vec<ulid::Ulid>,
	#[from_row(rename = "entitled_cache_invalidated_at")]
	pub invalidated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
	pub async fn into_old_model_partial(self, global: &Arc<Global>) -> Result<UserModelPartial, ()> {
		let connections: Vec<UserConnection> =
			scuffle_utils::database::query("SELECT * FROM user_connections WHERE user_id = $1")
				.bind(self.id)
				.build_query_as()
				.fetch_all(&global.db())
				.await
				.map_err(|_| ())?;
		let main_connection = connections.iter().find(|c| c.main_connection).ok_or(())?;

		let avatar_url = match self.active_cosmetics.profile_picture_id {
			Some(id) => global.file_by_id_loader().load(id).await?.map(|f| f.path),
			None => None,
		};

		let paint = match self.active_cosmetics.badge_id {
			Some(id) => match global.paint_by_id_loader().load(id).await? {
				Some(p) => Some(p.into_old_model(global).await?),
				None => None,
			},
			None => None,
		};

		let badge = match self.active_cosmetics.badge_id {
			Some(id) => match global.badge_by_id_loader().load(id).await? {
				Some(b) => Some(b.into_old_model(global).await?),
				None => None,
			},
			None => None,
		};

		let style = UserStyle {
			color: 0,
			paint_id: self.active_cosmetics.paint_id,
			paint,
			badge_id: self.active_cosmetics.badge_id,
			badge,
		};

		Ok(UserModelPartial {
			id: self.id,
			ty: String::new(),
			username: main_connection.platform_username.clone(),
			display_name: main_connection.platform_display_name.clone(),
			avatar_url,
			style,
			roles: self.entitled_cache.role_ids.into_iter().collect(),
			connections: connections.into_iter().map(UserConnectionPartial::from).collect(),
		})
	}

	pub async fn into_old_model(self, global: &Arc<Global>) -> Result<v3::types::User, ()> {
		let created_at = self.id.timestamp_ms();
		let partial = self.into_old_model_partial(global).await?;
		Ok(v3::types::User {
			id: partial.id,
			ty: partial.ty,
			username: partial.username,
			display_name: partial.display_name,
			created_at,
			avatar_url: partial.avatar_url,
			biography: String::new(),
			style: partial.style,
			emote_sets: todo!(),
			editors: todo!(),
			roles: partial.roles,
			connections: partial.connections.into_iter().map(|p| {
				v3::types::UserConnection {
					id: p.id,
					platform: p.platform,
					username: p.username,
					display_name: p.display_name,
					linked_at: p.linked_at,
					emote_capacity: p.emote_capacity,
					emote_set_id: p.emote_set_id,
					emote_set: todo!(),
					presences: todo!(),
					user: None,
				}
			}).collect(),
		})
	}
}
