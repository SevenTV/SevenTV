use super::TimeInterval;
use crate::database::Table;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProductPurchase {
	pub id: ulid::Ulid,
	pub product_id: ulid::Ulid,
	pub user_id: Option<ulid::Ulid>,
	pub data: ProductPurchaseData,
	pub status: ProductPurchaseStatus,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProductPurchaseData {
	#[serde(default)]
	pub was_gift: bool,
	#[serde(default)]
	/// If the product is a subscription product then this will be the amount of
	/// time the user purchased in this transaction.
	pub subscription_time: Option<TimeInterval>,
	pub price: f64,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum ProductPurchaseStatus {
	#[default]
	Pending,
	Completed,
	Cancelled,
	Refunded,
	Failed,
}

impl Table for ProductPurchase {
	const TABLE_NAME: &'static str = "product_purchases";
}
