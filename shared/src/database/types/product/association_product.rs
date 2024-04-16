use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct ProductAssociationProduct {
	pub product_id: ulid::Ulid,
	pub associated_product_id: ulid::Ulid,
}

impl Table for ProductAssociationProduct {
	const TABLE_NAME: &'static str = "product_association_product";
}
