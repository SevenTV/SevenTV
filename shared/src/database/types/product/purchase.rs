use postgres_types::{FromSql, ToSql};

use super::TimeInterval;
use crate::database::Table;

#[derive(Debug, Clone, postgres_from_row::FromRow)]
pub struct ProductPurchase {
	pub id: ulid::Ulid,
	pub product_id: ulid::Ulid,
	pub user_id: Option<ulid::Ulid>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
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

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "product_purchase_status")]
pub enum ProductPurchaseStatus {
	#[default]
	#[postgres(name = "PENDING")]
	Pending,
	#[postgres(name = "COMPLETED")]
	Completed,
	#[postgres(name = "CANCELLED")]
	Cancelled,
	#[postgres(name = "REFUNDED")]
	Refunded,
	#[postgres(name = "FAILED")]
	Failed,
}

impl Table for ProductPurchase {
	const TABLE_NAME: &'static str = "product_purchases";
}
