use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct ProductCodeAssociationProduct {
	pub product_code_id: ulid::Ulid,
	pub associated_product_id: ulid::Ulid,
}

impl Table for ProductCodeAssociationProduct {
	const TABLE_NAME: &'static str = "product_code_association_product";
}
