use super::UserId;
use crate::database::{Collection, Id, ProductCodeId, ProductId, ProductPurchaseId, TimeInterval};

pub type UserProductId = Id<UserProduct>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserProduct {
	#[serde(rename = "_id")]
	pub id: UserProductId,
	pub user_id: UserId,
	pub product_id: ProductId,
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
	pub duration: TimeInterval,
	pub created_by: UserProductDataPurchaseCreatedBy,
	pub start: chrono::DateTime<chrono::Utc>,
	pub end: chrono::DateTime<chrono::Utc>,
	pub status: UserProductDataSubscriptionEntryStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum UserProductDataSubscriptionEntryStatus {
	Active,
	Cancelled,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind", content = "value")]
pub enum UserProductDataPurchaseCreatedBy {
	Purchase(ProductPurchaseId),
	Code(ProductCodeId),
}

impl Collection for UserProduct {
	const COLLECTION_NAME: &'static str = "user_products";
}
