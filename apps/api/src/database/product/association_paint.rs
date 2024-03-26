use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct ProductAssociationPaint {
	pub product_id: ulid::Ulid,
	pub associated_paint_id: ulid::Ulid,
}

impl Table for ProductAssociationPaint {
	const TABLE_NAME: &'static str = "product_association_paint";
}
