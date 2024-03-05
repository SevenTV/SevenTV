use postgres_types::{FromSql, ToSql};

use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct ProductCode {
	pub id: ulid::Ulid,
	pub owner_id: Option<ulid::Ulid>,
	pub name: String,
	pub code: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub data: ProductCodeData,
	pub enabled: bool,
	pub remaining_uses: Option<i32>,
	pub kind: ProductCodeKind,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ProductCodeData {}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "product_code_kind")]
pub enum ProductCodeKind {
	#[default]
	#[postgres(name = "REDEEM")]
	Redeem,
	#[postgres(name = "DISCOUNT")]
	Discount,
	#[postgres(name = "GIFT")]
	Gift,
}

impl Table for ProductCode {
	const TABLE_NAME: &'static str = "product_code";
}
