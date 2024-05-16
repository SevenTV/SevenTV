use super::Collection;

mod purchase;
mod invoice;

// An item that can be purchased
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
	// This ID will be the stripe ID for the product
	#[serde(rename = "_id")]
	pub id: stripe::ProductId,
	pub kind: ProductKind,
	// there will be other fields here like name, description, price, etc.
	// those fields will be shown in the UI but are not relevant to the core logic
	// We should also make those fields sync from Stripe.
	pub prices: Vec<ProductPrice>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductPrice {
	pub id: stripe::PriceId,
	// some other fields like currency, amount, etc.
}

// The kind of product
#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum ProductKind {
	Subscription = 0,
	OneTimePurchase = 1,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductRef {
	// The invoice id
	pub id: stripe::InvoiceId,
	// The item this reference refers to otherwise it is the whole invoice
	pub price_id: stripe::PriceId,
}

impl Collection for Product {
	const COLLECTION_NAME: &'static str = "products";
}
