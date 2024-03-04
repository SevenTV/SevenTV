use postgres_types::{FromSql, ToSql};

use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct ProductSubscription {
	pub product_id: ulid::Ulid,
	pub user_id: ulid::Ulid,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: ProductSubscriptionData,
	pub status: ProductSubscriptionStatus,
	pub next_payment_due: Option<chrono::DateTime<chrono::Utc>>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ProductSubscriptionData {}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "product_subscription_status")]
pub enum ProductSubscriptionStatus {
	#[default]
	#[postgres(name = "ACTIVE")]
	Active,
	#[postgres(name = "PENDING")]
	Pending,
	#[postgres(name = "CANCELLED")]
	Cancelled,
	#[postgres(name = "EXPIRED")]
	Expired,
}

impl Table for ProductSubscription {
	const TABLE_NAME: &'static str = "product_subscriptions";
}
