use postgres_types::{FromSql, ToSql};

use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct ProductPurchase {
	pub id: ulid::Ulid,
	pub product_id: ulid::Ulid,
	pub user_id: Option<ulid::Ulid>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: ProductPurchaseData,
	pub status: ProductPurchaseStatus,
	pub gift_code_id: Option<ulid::Ulid>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ProductPurchaseData {}

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
