use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct ProductAssociationBadge {
	pub product_id: ulid::Ulid,
	pub associated_badge_id: ulid::Ulid,
}

impl Table for ProductAssociationBadge {
	const TABLE_NAME: &'static str = "product_association_badge";
}
