use std::collections::HashSet;

use chrono::Utc;

use crate::database::product::codes::DiscountCodeId;
use crate::database::product::invoice::InvoiceStatus;
use crate::database::product::{CustomerId, InvoiceId, ProductId};
use crate::database::user::UserId;
use crate::database::{self};
use crate::typesense::types::{TypesenseCollection, TypesenseGenericCollection};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypesenseCollection)]
#[typesense(collection_name = "invoices")]
#[serde(deny_unknown_fields)]
pub struct Invoice {
	pub id: InvoiceId,
	pub items: Vec<ProductId>,
	pub discount_codes: Vec<DiscountCodeId>,
	pub customer_id: CustomerId,
	pub user_id: UserId,
	pub paypal_payment_ids: Vec<String>,
	pub status: InvoiceStatus,
	pub note: Option<String>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::product::invoice::Invoice> for Invoice {
	fn from(invoice: database::product::invoice::Invoice) -> Self {
		Self {
			id: invoice.id,
			discount_codes: invoice
				.items
				.iter()
				.flat_map(|item| item.discount_codes.iter())
				.cloned()
				.collect::<HashSet<_>>()
				.into_iter()
				.collect(),
			items: invoice.items.into_iter().map(|item| item.product_id).collect(),
			customer_id: invoice.customer_id,
			user_id: invoice.user_id,
			paypal_payment_ids: invoice.paypal_payment_ids,
			status: invoice.status,
			note: invoice.note,
			created_at: invoice.created_at.timestamp_millis(),
			updated_at: invoice.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<Invoice>()]
}
