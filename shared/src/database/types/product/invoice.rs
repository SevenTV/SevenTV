use super::{CustomerId, InvoiceId, ProductId};
use crate::database::types::MongoGenericCollection;
use crate::database::user::UserId;
use crate::database::MongoCollection;
use crate::typesense::types::impl_typesense_type;

/// Only for showing to the user.
/// Technically not necessary.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "invoices")]
#[mongo(index(fields(user_id = 1)))]
#[mongo(index(fields(paypal_payment_id = 1)))]
#[mongo(index(fields(items = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(search = "crate::typesense::types::product::invoice::Invoice")]
#[serde(deny_unknown_fields)]
pub struct Invoice {
	/// This ID will be the stripe ID for the invoice
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: InvoiceId,
	/// These items will be the stripe line items for the invoice
	pub items: Vec<ProductId>,
	/// customer id from stripe
	pub customer_id: CustomerId,
	/// User who the invoice is for
	pub user_id: UserId,
	/// If this invoice was paid via a legacy payment
	pub paypal_payment_id: Option<String>,
	/// Status of the invoice
	pub status: InvoiceStatus,
	/// If a payment failed
	pub failed: bool,
	/// If the invoice was refunded
	pub refunded: bool,
	/// If the invoice was disputed
	pub disputed: Option<InvoiceDisputeStatus>,
	#[serde(with = "crate::database::serde")]
	/// Created at
	pub created_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	/// Updated at
	pub updated_at: chrono::DateTime<chrono::Utc>,
	/// Search updated at
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum InvoiceStatus {
	Draft = 0,
	Open = 1,
	Paid = 2,
	Uncollectible = 3,
	Void = 4,
}

impl_typesense_type!(InvoiceStatus, Int32);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum InvoiceDisputeStatus {
	Lost = 0,
	NeedsResponse = 1,
	UnderReview = 2,
	WarningClosed = 3,
	WarningNeedsResponse = 4,
	WarningUnderReview = 5,
	Won = 6,
	/// only applies to paypal disputes, either won or lost, we don't know
	Resolved = 7,
}

impl From<stripe::DisputeStatus> for InvoiceDisputeStatus {
	fn from(value: stripe::DisputeStatus) -> Self {
		match value {
			stripe::DisputeStatus::Lost => InvoiceDisputeStatus::Lost,
			stripe::DisputeStatus::NeedsResponse => InvoiceDisputeStatus::NeedsResponse,
			stripe::DisputeStatus::UnderReview => InvoiceDisputeStatus::UnderReview,
			stripe::DisputeStatus::WarningClosed => InvoiceDisputeStatus::WarningClosed,
			stripe::DisputeStatus::WarningNeedsResponse => InvoiceDisputeStatus::WarningNeedsResponse,
			stripe::DisputeStatus::WarningUnderReview => InvoiceDisputeStatus::WarningUnderReview,
			stripe::DisputeStatus::Won => InvoiceDisputeStatus::Won,
		}
	}
}

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<Invoice>()]
}
