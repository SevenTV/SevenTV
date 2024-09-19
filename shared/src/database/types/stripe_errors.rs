use macros::MongoCollection;

use crate::database::Id;

pub type StripeErrorId = Id<StripeError>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection, PartialEq)]
#[mongo(collection_name = "stripe_errors")]
#[serde(deny_unknown_fields)]
pub struct StripeError {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: StripeErrorId,
	pub event_id: stripe::EventId,
	pub error_kind: StripeErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum StripeErrorKind {
	/// The subscription contains mutiple products
	SubscriptionMultipleProducts = 0,
	/// The subscription item changed
	SubscriptionItemChanged = 1,
	/// The subscription invoice contains multiple or no line items
	SubscriptionInvoiceInvalidItems = 2,
	/// No subscription product was found for the invoice
	SubscriptionInvoiceNoProduct = 3,
	/// The gift invoice contains no line items
	GiftInvoiceNoProduct = 4,
}
