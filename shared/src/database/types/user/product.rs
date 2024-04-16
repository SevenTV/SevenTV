use crate::database::{Table, TimeInterval};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct UserProduct {
	pub user_id: ulid::Ulid,
	pub product_id: ulid::Ulid,
	pub data: UserProductData,
	pub created_at: chrono::DateTime<chrono::Utc>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct UserProductData {
	pub purchases: Vec<UserProductDataPurchase>,
	pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[serde(tag = "kind", content = "value")]
pub enum UserProductDataPurchaseCreatedBy {
	Purchase(ulid::Ulid),
	Code(ulid::Ulid),
}

impl Table for UserProduct {
	const TABLE_NAME: &'static str = "user_products";
}

pub struct UserProductPurchaseAssociation {
	pub user_id: ulid::Ulid,
	pub product_id: ulid::Ulid,
	pub product_purchase_id: ulid::Ulid,
}

impl Table for UserProductPurchaseAssociation {
	const TABLE_NAME: &'static str = "user_product_purchase_association";
}

pub struct UserProductCodeAssociation {
	pub user_id: ulid::Ulid,
	pub product_id: ulid::Ulid,
	pub product_code_id: ulid::Ulid,
}

impl Table for UserProductCodeAssociation {
	const TABLE_NAME: &'static str = "user_product_gift_association";
}
