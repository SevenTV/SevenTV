use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::codes::{DiscountCodeId, GiftCodeId, RedeemCodeId};
use super::subscription::SubscriptionPeriodId;
use super::{InvoiceId, InvoiceLineItemId, SubscriptionId};
use crate::database::types::GenericCollection;
use crate::database::{Collection, Id};

/// https://www.mermaidchart.com/raw/e2b0bfc4-eaa4-4eca-834c-d5b9ad787021?version=v0.1&format=svg
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProductEdgeKind {
	InvoiceLineItem {
		invoice_id: InvoiceId,
		line_item_id: InvoiceLineItemId,
	},
	SubscriptionPeriod {
		subscription_id: SubscriptionId,
		period_id: SubscriptionPeriodId,
	},
	GiftCode {
		gift_code_id: GiftCodeId,
	},
	RedeemCode {
		redeem_code_id: RedeemCodeId,
	},
	DiscountCode {
		discount_code_id: DiscountCodeId,
	},
}

impl ProductEdgeKind {
	pub fn has_inbound(&self) -> bool {
		matches!(
			self,
			Self::InvoiceLineItem { .. } | Self::SubscriptionPeriod { .. } | Self::GiftCode { .. }
		)
	}

	pub fn has_outbound(&self) -> bool {
		matches!(
			self,
			Self::RedeemCode { .. } | Self::DiscountCode { .. } | Self::InvoiceLineItem { .. } | Self::GiftCode { .. }
		)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
/// A product edge is used to determine dependencies between invoice line items
/// and whatever they end up being used for.
pub struct ProductEdge {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: Id<ProductEdge>,
	pub from: ProductEdgeKind,
	pub to: ProductEdgeKind,
}

impl ProductEdge {
	pub fn new(from: ProductEdgeKind, to: ProductEdgeKind) -> Self {
		Self {
			id: Id::default(),
			from,
			to,
		}
	}
}

impl Collection for ProductEdge {
	const COLLECTION_NAME: &'static str = "product_edges";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"from": 1,
					"to": 1,
				})
				.options(mongodb::options::IndexOptions::builder().unique(true).build())
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"to": 1,
					"from": 1,
				})
				.build(),
		]
	}
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<ProductEdge>()]
}
