use bson::oid::ObjectId;

use crate::database::Collection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProductCode {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub owner_id: Option<ObjectId>,
	pub purchase_id: Option<ObjectId>,
	pub name: String,
	pub code: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub enabled: bool,
	pub remaining_uses: Option<i32>,
	pub kind: ProductCodeKind,
	pub product_ids: Vec<ObjectId>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum ProductCodeKind {
	#[default]
	Redeem,
	Discount,
	Gift,
}

impl Collection for ProductCode {
	const NAME: &'static str = "product_codes";
}
