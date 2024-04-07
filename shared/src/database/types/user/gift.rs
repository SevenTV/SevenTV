use postgres_types::{FromSql, ToSql};

use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserGift {
	pub id: ulid::Ulid,
	pub sender_id: Option<ulid::Ulid>,
	pub recipient_id: ulid::Ulid,
	pub product_code_id: ulid::Ulid,
	pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub status: UserGiftStatus,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: UserGiftData,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct UserGiftData {
	pub message: Option<String>,
	/// If the gift was given to the recipient by 7TV itself, this will be true.
	/// Meaning nobody actually bought the gift for the recipient.
	pub system: bool,
}

impl Table for UserGift {
	const TABLE_NAME: &'static str = "user_gifts";
}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "user_gift_status")]
pub enum UserGiftStatus {
	#[default]
	#[postgres(name = "ACTIVE")]
	Active,
	#[postgres(name = "REDEEMED")]
	Redeemed,
	#[postgres(name = "EXPIRED")]
	Expired,
	#[postgres(name = "CANCELLED")]
	Cancelled,
}
