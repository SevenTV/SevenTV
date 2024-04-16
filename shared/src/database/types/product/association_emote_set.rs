use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct ProductAssociationEmoteSet {
	pub product_id: ulid::Ulid,
	pub associated_emote_set_id: ulid::Ulid,
}

impl Table for ProductAssociationEmoteSet {
	const TABLE_NAME: &'static str = "product_association_emote_set";
}
