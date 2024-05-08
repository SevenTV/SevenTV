use super::UserId;
use crate::database::{Collection, GiftCodeId, Id, PriceId, PurchaseId, RedeemCodeId};

pub type UserProductId = Id<UserProduct>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserProduct {
	#[serde(rename = "_id")]
	pub id: UserProductId,
	pub user_id: UserId,
	pub product_id: PriceId,
	pub data: UserProductData,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserProductData {
	pub purchases: Vec<UserProductDataPurchase>,
	pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserProductDataPurchase {
	// pub duration: TimeInterval,
	pub created_by: UserProductDataPurchaseCreatedBy,
	pub start: chrono::DateTime<chrono::Utc>,
	pub end: chrono::DateTime<chrono::Utc>,
	pub status: UserProductDataSubscriptionEntryStatus,
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr, PartialEq, Eq)]
#[repr(u8)]
pub enum UserProductDataSubscriptionEntryStatus {
	Active = 0,
	Cancelled = 1,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind", content = "value")]
pub enum UserProductDataPurchaseCreatedBy {
	Purchase(PurchaseId),
	Redeem(RedeemCodeId),
	Gift(GiftCodeId),
}

impl Collection for UserProduct {
	const COLLECTION_NAME: &'static str = "user_products";
}
