use super::UserId;
use crate::database::{Collection, Id, ProductCodeId};

pub type UserGiftId = Id<UserGift>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserGift {
	#[serde(rename = "_id", with = "crate::database::id::bson")]
	pub id: UserGiftId,
	pub sender_id: Option<UserId>,
	pub recipient_id: UserId,
	pub product_code_id: ProductCodeId,
	pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
	pub status: UserGiftStatus,
	pub message: Option<String>,
	/// If the gift was given to the recipient by 7TV itself, this will be true.
	/// Meaning nobody actually bought the gift for the recipient.
	pub system: bool,
}

impl Collection for UserGift {
	const COLLECTION_NAME: &'static str = "user_gifts";
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum UserGiftStatus {
	#[default]
	Active,
	Redeemed,
	Expired,
	Cancelled,
}
