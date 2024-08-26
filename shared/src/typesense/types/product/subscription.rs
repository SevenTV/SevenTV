use chrono::Utc;

use crate::database::product::codes::{GiftCodeId, RedeemCodeId};
use crate::database::product::subscription::SubscriptionPeriodId;
use crate::database::product::{InvoiceId, ProductId, SubscriptionId};
use crate::database::user::UserId;
use crate::database::{self};
use crate::typesense::types::{impl_typesense_type, TypesenseCollection, TypesenseGenericCollection};

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum SubscriptionCreatedByKind {
	RedeemCode = 0,
	GiftCode = 1,
	Invoice = 2,
	System = 3,
}

impl SubscriptionCreatedByKind {
	fn split(
		created_by: database::product::subscription::SubscriptionPeriodCreatedBy,
	) -> (
		SubscriptionCreatedByKind,
		Option<RedeemCodeId>,
		Option<GiftCodeId>,
		Option<InvoiceId>,
		Option<String>,
	) {
		match created_by {
			database::product::subscription::SubscriptionPeriodCreatedBy::RedeemCode { redeem_code_id } => (
				SubscriptionCreatedByKind::RedeemCode,
				Some(redeem_code_id),
				None,
				None,
				None,
			),
			database::product::subscription::SubscriptionPeriodCreatedBy::GiftCode { gift_code_id } => (
				SubscriptionCreatedByKind::GiftCode,
				None,
				Some(gift_code_id),
				None,
				None,
			),
			database::product::subscription::SubscriptionPeriodCreatedBy::Invoice {
				invoice_id,
			} => (
				SubscriptionCreatedByKind::Invoice,
				None,
				None,
				Some(invoice_id),
				None,
			),
			database::product::subscription::SubscriptionPeriodCreatedBy::System { reason } => {
				(SubscriptionCreatedByKind::System, None, None, None, reason)
			}
		}
	}
}

impl_typesense_type!(SubscriptionCreatedByKind, Int32);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypesenseCollection)]
#[typesense(collection_name = "subscription_periods")]
#[serde(deny_unknown_fields)]
pub struct SubscriptionPeriod {
	pub id: SubscriptionPeriodId,
	pub subscription_id: SubscriptionId,
	pub user_id: UserId,
	pub start: i64,
	pub end: i64,
	pub created_by_kind: SubscriptionCreatedByKind,
	pub created_by_redeem_code_id: Option<RedeemCodeId>,
	pub created_by_gift_code_id: Option<GiftCodeId>,
	pub created_by_invoice_id: Option<InvoiceId>,
	pub created_by_system_reason: Option<String>,
	pub product_ids: Vec<ProductId>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<crate::database::product::subscription::SubscriptionPeriod> for SubscriptionPeriod {
	fn from(value: crate::database::product::subscription::SubscriptionPeriod) -> Self {
		let (
			created_by_kind,
			created_by_redeem_code_id,
			created_by_gift_code_id,
			created_by_invoice_id,
			created_by_system_reason,
		) = SubscriptionCreatedByKind::split(value.created_by);

		Self {
			id: value.id,
			subscription_id: value.subscription_id,
			user_id: value.user_id,
			start: value.start.timestamp_millis(),
			end: value.end.timestamp_millis(),
			product_ids: value.product_ids,
			created_by_kind,
			created_by_redeem_code_id,
			created_by_gift_code_id,
			created_by_invoice_id,
			created_by_system_reason,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[
		TypesenseGenericCollection::new::<SubscriptionPeriod>(),
	]
}
