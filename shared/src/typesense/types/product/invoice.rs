use chrono::Utc;

use crate::database::product::invoice::InvoiceStatus;
use crate::database::product::{CustomerId, InvoiceId, StripeProductId};
use crate::database::user::UserId;
use crate::database::{self};
use crate::typesense::types::{TypesenseCollection, TypesenseGenericCollection};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypesenseCollection)]
#[typesense(collection_name = "invoices")]
#[serde(deny_unknown_fields)]
pub struct Invoice {
	pub id: InvoiceId,
	pub items: Vec<StripeProductId>,
	pub customer_id: CustomerId,
	pub user_id: UserId,
	pub paypal_payment_id: Option<String>,
	pub status: InvoiceStatus,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::product::invoice::Invoice> for Invoice {
	fn from(invoice: database::product::invoice::Invoice) -> Self {
		Self {
			id: invoice.id,
			items: invoice.items,
			customer_id: invoice.customer_id,
			user_id: invoice.user_id,
			paypal_payment_id: invoice.paypal_payment_id,
			status: invoice.status,
			created_at: invoice.created_at.timestamp_millis(),
			updated_at: invoice.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<Invoice>()]
}
