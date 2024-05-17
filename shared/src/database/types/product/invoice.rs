use crate::database::{Collection, UserId};

use super::ProductRef;

pub type InvoiceId = stripe::InvoiceId;

// An invoice that is generated for a purchase
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invoice {
	// This ID will be the stripe ID for the invoice
	#[serde(rename = "_id")]
	pub id: InvoiceId,
	// These items will be the stripe line items for the invoice
	pub items: Vec<InvoiceItem>,
	// customer id from stripe
	pub customer_id: stripe::CustomerId,
	// User who the invoice is for
	pub user_id: UserId,
	// If this invoice was paid via a legacy payment
	pub paypal_payment_ids: Vec<String>,
	// If the invoice was deleted
	pub status: InvoiceStatus,
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum InvoiceStatus {
	Draft = 0,
    Open = 1,
    Paid = 2,
    Uncollectible = 3,
    Void = 4,
}

impl From<InvoiceStatus> for stripe::InvoiceStatus {
	fn from(value: InvoiceStatus) -> Self {
		match value {
			InvoiceStatus::Draft => stripe::InvoiceStatus::Draft,
			InvoiceStatus::Open => stripe::InvoiceStatus::Open,
			InvoiceStatus::Paid => stripe::InvoiceStatus::Paid,
			InvoiceStatus::Uncollectible => stripe::InvoiceStatus::Uncollectible,
			InvoiceStatus::Void => stripe::InvoiceStatus::Void,
		}
	}
}

impl From<stripe::InvoiceStatus> for InvoiceStatus {
	fn from(value: stripe::InvoiceStatus) -> Self {
		match value {
			stripe::InvoiceStatus::Draft => InvoiceStatus::Draft,
			stripe::InvoiceStatus::Open => InvoiceStatus::Open,
			stripe::InvoiceStatus::Paid => InvoiceStatus::Paid,
			stripe::InvoiceStatus::Uncollectible => InvoiceStatus::Uncollectible,
			stripe::InvoiceStatus::Void => InvoiceStatus::Void,
		}
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvoiceItem {
	// This will be a line item id from stripe
	pub id: stripe::InvoiceLineItemId,
	// This is a stripe id for the product
	pub product: ProductRef,
	// User who the item is for
	pub user_id: UserId,
	// Other information about the item like quantity, price, subscription plan, etc.
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvoiceRef {
	// The invoice id
	pub id: stripe::InvoiceId,
	// Optionally the item this reference refers to otherwise it is the whole invoice
	pub item_id: stripe::InvoiceLineItemId,
}

impl Collection for Invoice {
    const COLLECTION_NAME: &'static str = "invoices";
}
