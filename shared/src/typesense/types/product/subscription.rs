use chrono::Utc;

use crate::database::entitlement::{EntitlementEdgeKind, EntitlementEdgeKindString};
use crate::database::product::codes::RedeemCodeId;
use crate::database::product::subscription::{ProviderSubscriptionId, SubscriptionPeriodId, SubscriptionState};
use crate::database::product::{InvoiceId, SubscriptionProductId};
use crate::database::user::UserId;
use crate::database::{self};
use crate::typesense::types::{impl_typesense_type, TypesenseCollection, TypesenseGenericCollection};

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum SubscriptionCreatedByKind {
	RedeemCode = 0,
	Gift = 1,
	Invoice = 2,
	System = 3,
}

impl SubscriptionCreatedByKind {
	fn split(
		created_by: database::product::subscription::SubscriptionPeriodCreatedBy,
	) -> (
		SubscriptionCreatedByKind,
		Option<RedeemCodeId>,
		Option<InvoiceId>,
		Option<String>,
	) {
		match created_by {
			database::product::subscription::SubscriptionPeriodCreatedBy::RedeemCode { redeem_code_id, .. } => {
				(SubscriptionCreatedByKind::RedeemCode, Some(redeem_code_id), None, None)
			}
			database::product::subscription::SubscriptionPeriodCreatedBy::Invoice { invoice_id, .. } => {
				(SubscriptionCreatedByKind::Invoice, None, Some(invoice_id), None)
			}
			database::product::subscription::SubscriptionPeriodCreatedBy::System { reason } => {
				(SubscriptionCreatedByKind::System, None, None, reason)
			}
			database::product::subscription::SubscriptionPeriodCreatedBy::Gift { .. } => {
				(SubscriptionCreatedByKind::Gift, None, None, None)
			}
		}
	}
}

impl_typesense_type!(SubscriptionCreatedByKind, Int32);

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum SubscriptionProvider {
	Stripe = 0,
	Paypal = 1,
}

impl_typesense_type!(SubscriptionProvider, Int32);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypesenseCollection)]
#[typesense(collection_name = "subscription_periods")]
#[serde(deny_unknown_fields)]
pub struct SubscriptionPeriod {
	pub id: SubscriptionPeriodId,

	pub user_id: UserId,
	pub product_id: SubscriptionProductId,

	pub subscription_provider: Option<SubscriptionProvider>,
	pub provider_id: Option<String>,
	pub start: i64,
	pub end: i64,
	pub created_by_kind: SubscriptionCreatedByKind,
	pub created_by_redeem_code_id: Option<RedeemCodeId>,
	pub created_by_invoice_id: Option<InvoiceId>,
	pub created_by_system_reason: Option<String>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<crate::database::product::subscription::SubscriptionPeriod> for SubscriptionPeriod {
	fn from(value: crate::database::product::subscription::SubscriptionPeriod) -> Self {
		let (created_by_kind, created_by_redeem_code_id, created_by_invoice_id, created_by_system_reason) =
			SubscriptionCreatedByKind::split(value.created_by);

		Self {
			id: value.id,
			subscription_provider: match value.provider_id {
				Some(ProviderSubscriptionId::Stripe(_)) => Some(SubscriptionProvider::Stripe),
				Some(ProviderSubscriptionId::Paypal(_)) => Some(SubscriptionProvider::Paypal),
				None => None,
			},
			provider_id: value.provider_id.map(|id| id.to_string()),
			user_id: value.subscription_id.user_id,
			product_id: value.subscription_id.product_id,
			start: value.start.timestamp_millis(),
			end: value.end.timestamp_millis(),
			created_by_kind,
			created_by_redeem_code_id,
			created_by_invoice_id,
			created_by_system_reason,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypesenseCollection)]
#[typesense(collection_name = "subscriptions")]
#[serde(deny_unknown_fields)]
pub struct Subscription {
	pub id: String,
	pub user_id: UserId,
	pub product_id: SubscriptionProductId,
	pub state: SubscriptionState,
	pub entitlements: Vec<EntitlementEdgeKindString>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub ended_at: Option<i64>,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl Subscription {
	pub fn from_db(
		value: crate::database::product::subscription::Subscription,
		entitlements: impl IntoIterator<Item = EntitlementEdgeKind>,
	) -> Self {
		Self {
			id: value.id.to_string(),
			user_id: value.id.user_id,
			product_id: value.id.product_id,
			state: value.state,
			entitlements: entitlements.into_iter().map(Into::into).collect(),
			created_at: value.created_at.timestamp_millis(),
			ended_at: value.ended_at.map(|t| t.timestamp_millis()),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[
		TypesenseGenericCollection::new::<SubscriptionPeriod>(),
		TypesenseGenericCollection::new::<Subscription>(),
	]
}
