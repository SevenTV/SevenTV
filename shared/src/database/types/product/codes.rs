use serde::{Deserialize, Serialize};

use super::{SubscriptionProductId, TimePeriod};
use crate::database::entitlement::EntitlementEdgeKind;
use crate::database::types::MongoGenericCollection;
use crate::database::user::UserId;
use crate::database::{Id, MongoCollection};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CodeEffect {
	Entitlement { edge: EntitlementEdgeKind, extends_subscription: Option<SubscriptionProductId> },
	SubscriptionProduct { id: SubscriptionProductId, trial_days: u32 },
}

// impl std::fmt::Display for CodeEffect {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		match self {
// 			CodeEffect::Entitlement { edge } => {
// 				write!(f, "entitlement:{edge}",)
// 			}
// 			CodeEffect::SubscriptionProduct { id, .. } => {
// 				write!(f, "subscription_product:{id}")
// 			}
// 		}
// 	}
// }

pub type RedeemCodeId = Id<RedeemCode>;

#[derive(Debug, Clone, Serialize, Deserialize, MongoCollection)]
#[mongo(collection_name = "redeem_codes")]
#[mongo(index(fields(code = 1), unique))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
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
	pub active_period: TimePeriod,
	pub effects: Vec<CodeEffect>,
	pub created_by: UserId,
	pub special_event_id: Option<SpecialEventId>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub type SpecialEventId = Id<SpecialEvent>;

#[derive(Debug, Clone, Serialize, Deserialize, MongoCollection)]
#[mongo(collection_name = "special_events")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct SpecialEvent {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SpecialEventId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub created_by: UserId,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

// pub type DiscountCodeId = Id<DiscountCode>;

// #[derive(Debug, Clone, Serialize, Deserialize, MongoCollection)]
// #[mongo(collection_name = "discount_codes")]
// #[mongo(index(fields(code = 1), unique))]
// #[mongo(index(fields(search_updated_at = 1)))]
// #[mongo(index(fields(_id = 1, updated_at = -1)))]
// #[serde(deny_unknown_fields)]
// pub struct DiscountCode {
// 	#[mongo(id)]
// 	#[serde(rename = "_id")]
// 	pub id: DiscountCodeId,
// 	pub name: String,
// 	pub description: Option<String>,
// 	pub tags: Vec<String>,
// 	pub code: String,
// 	pub discount: Discount,
// 	pub active_period: TimePeriod,
// 	pub max_uses: Option<i32>,
// 	pub remaining_uses: Option<i32>,
// 	pub max_uses_per_user: Option<i32>,
// 	pub product_ids: Vec<ProductId>,
// 	pub created_by: UserId,
// 	#[serde(with = "crate::database::serde")]
// 	pub updated_at: chrono::DateTime<chrono::Utc>,
// 	#[serde(with = "crate::database::serde")]
// 	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(tag = "type")]
// pub enum Discount {
// 	Percentage { percentage: f64 },
// 	Fixed { amount: f64, currency: stripe::Currency },
// }

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[
		MongoGenericCollection::new::<RedeemCode>(),
		// MongoGenericCollection::new::<DiscountCode>(),
		MongoGenericCollection::new::<SpecialEvent>(),
	]
}
