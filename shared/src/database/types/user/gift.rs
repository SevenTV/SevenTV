use bson::oid::ObjectId;

use crate::database::Collection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserGift {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub sender_id: Option<ObjectId>,
	pub recipient_id: ObjectId,
	pub product_code_id: ObjectId,
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
