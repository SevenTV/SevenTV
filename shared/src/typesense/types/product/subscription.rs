use chrono::Utc;

use crate::database::product::codes::{GiftCodeId, RedeemCodeId};
use crate::database::product::subscription::{SubscriptionCreditId, SubscriptionPeriodId};
use crate::database::product::{InvoiceId, InvoiceLineItemId, ProductId, SubscriptionId};
use crate::database::user::UserId;
use crate::database::{self};
use crate::typesense::types::duration_unit::DurationUnit;
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
		created_by: database::product::subscription::SubscriptionCreatedBy,
	) -> (
		SubscriptionCreatedByKind,
		Option<RedeemCodeId>,
		Option<GiftCodeId>,
		Option<InvoiceId>,
		Option<InvoiceLineItemId>,
		Option<String>,
	) {
		match created_by {
			database::product::subscription::SubscriptionCreatedBy::RedeemCode { redeem_code_id } => (
				SubscriptionCreatedByKind::RedeemCode,
				Some(redeem_code_id),
				None,
				None,
				None,
				None,
			),
			database::product::subscription::SubscriptionCreatedBy::GiftCode { gift_code_id } => (
				SubscriptionCreatedByKind::GiftCode,
				None,
				Some(gift_code_id),
				None,
				None,
				None,
			),
			database::product::subscription::SubscriptionCreatedBy::Invoice {
				invoice_id,
				invoice_item_id,
			} => (
				SubscriptionCreatedByKind::Invoice,
				None,
				None,
				Some(invoice_id),
				Some(invoice_item_id),
				None,
			),
			database::product::subscription::SubscriptionCreatedBy::System { reason } => {
				(SubscriptionCreatedByKind::System, None, None, None, None, reason)
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
	pub created_by_invoice_item_id: Option<InvoiceLineItemId>,
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
			created_by_invoice_item_id,
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
			created_by_invoice_item_id,
			created_by_system_reason,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypesenseCollection)]
#[typesense(collection_name = "subscription_credits")]
#[serde(deny_unknown_fields)]
pub struct SubscriptionCredit {
	pub id: SubscriptionCreditId,
	pub user_id: UserId,
	pub duration_kind: DurationUnit,
	pub duration_value: i32,
	pub created_by_kind: SubscriptionCreatedByKind,
	pub created_by_redeem_code_id: Option<RedeemCodeId>,
	pub created_by_gift_code_id: Option<GiftCodeId>,
	pub created_by_invoice_id: Option<InvoiceId>,
	pub created_by_invoice_item_id: Option<InvoiceLineItemId>,
	pub created_by_system_reason: Option<String>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<crate::database::product::subscription::SubscriptionCredit> for SubscriptionCredit {
	fn from(value: crate::database::product::subscription::SubscriptionCredit) -> Self {
		let (
			created_by_kind,
			created_by_redeem_code_id,
			created_by_gift_code_id,
			created_by_invoice_id,
			created_by_invoice_item_id,
			created_by_system_reason,
		) = SubscriptionCreatedByKind::split(value.created_by);

		let (duration_kind, duration_value) = DurationUnit::split(value.duration);

		Self {
			id: value.id,
			user_id: value.user_id,
			duration_kind,
			duration_value,
			created_by_kind,
			created_by_redeem_code_id,
			created_by_gift_code_id,
			created_by_invoice_id,
			created_by_invoice_item_id,
			created_by_system_reason,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[
		TypesenseGenericCollection::new::<SubscriptionCredit>(),
		TypesenseGenericCollection::new::<SubscriptionPeriod>(),
	]
}
