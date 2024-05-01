use super::{ProductId, ProductPurchaseId};
use crate::database::{Collection, Id, UserId};

pub type ProductCodeId = Id<ProductCode>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProductCode {
	#[serde(rename = "_id")]
	pub id: ProductCodeId,
	pub owner_id: Option<UserId>,
	pub purchase_id: Option<ProductPurchaseId>,
	pub name: String,
	pub code: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub enabled: bool,
	pub remaining_uses: Option<i32>,
	pub kind: ProductCodeKind,
	pub product_ids: Vec<ProductId>,
}

#[derive(Debug, Clone, Default, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum ProductCodeKind {
	#[default]
	Redeem = 0,
	Discount = 1,
	Gift = 2,
}

impl Collection for ProductCode {
	const COLLECTION_NAME: &'static str = "product_codes";
}
