use serde::{Deserialize, Serialize};

use super::special_event::SpecialEventId;
use super::{SubscriptionProductId, TimePeriod};
use crate::database::entitlement::EntitlementEdgeKind;
use crate::database::types::MongoGenericCollection;
use crate::database::user::UserId;
use crate::database::{Id, MongoCollection};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CodeEffect {
	/// Codes that give entitlements directly to the user or attached to their
	/// subscription.
	DirectEntitlement { entitlements: Vec<EntitlementEdgeKind> },
	/// Codes that are part of a special event, where we will create a Edge
	/// between the user (or their subscription) and the special event.
	/// Entitlements will then be attached to the special event.
	SpecialEvent { special_event_id: SpecialEventId },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedeemCodeSubscriptionEffect {
	pub id: SubscriptionProductId,
	/// If not set then the sub will not be given to the user if they do not
	/// have a sub. However the entitlements will still require a sub to be
	/// active.
	pub trial_days: Option<i32>,
	/// Redirect to stripe checkout (only if trial_days is set, force the user to subscribe if not then the paint will be added to their future sub whenever they subscribe)
	#[serde(default)]
	pub no_redirect_to_stripe: bool,
}

pub type RedeemCodeId = Id<RedeemCode>;

#[derive(Debug, Clone, Serialize, Deserialize, MongoCollection)]
#[mongo(collection_name = "redeem_codes")]
#[mongo(index(fields(code = 1), unique))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(search = "crate::typesense::types::product::codes::RedeemCode")]
#[serde(deny_unknown_fields)]
pub struct RedeemCode {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: RedeemCodeId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub code: String,
	pub remaining_uses: i32,
	pub active_period: Option<TimePeriod>,
	pub subscription_effect: Option<RedeemCodeSubscriptionEffect>,
	pub created_by: UserId,
	pub effect: CodeEffect,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<RedeemCode>()]
}
