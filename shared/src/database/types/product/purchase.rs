use bson::oid::ObjectId;
use bson::Bson;

use super::TimeInterval;
use crate::database::Collection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProductPurchase {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub product_id: ObjectId,
	pub user_id: Option<ObjectId>,
	pub was_gift: bool,
	/// If the product is a subscription product then this will be the amount of
	/// time the user purchased in this transaction.
	pub subscription_time: Option<TimeInterval>,
	pub price: f64,
	pub status: ProductPurchaseStatus,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProductPurchaseStatus {
	#[default]
	Pending,
	Completed,
	Cancelled,
	Refunded,
	Failed,
}

impl Collection for ProductPurchase {
	const COLLECTION_NAME: &'static str = "product_purchases";
}
