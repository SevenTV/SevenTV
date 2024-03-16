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
mod settings;

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
pub use settings::*;

use crate::database::Table;

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
