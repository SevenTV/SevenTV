use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct UserGift {
	pub id: ulid::Ulid,
	pub sender_id: Option<ulid::Ulid>,
	pub recipient_id: ulid::Ulid,
	pub product_code_id: ulid::Ulid,
	pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub status: UserGiftStatus,
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

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum UserGiftStatus {
	#[default]
	Active,
	Redeemed,
	Expired,
	Cancelled,
}
