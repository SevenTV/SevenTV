use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct ProductAssociationRole {
	pub product_id: ulid::Ulid,
	pub associated_role_id: ulid::Ulid,
}

impl Table for ProductAssociationRole {
	const TABLE_NAME: &'static str = "product_association_role";
}
