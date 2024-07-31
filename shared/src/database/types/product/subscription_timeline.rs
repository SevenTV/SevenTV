use super::{ProductId, TimePeriod};
use crate::database::duration::DurationUnit;
use crate::database::types::MongoGenericCollection;
use crate::database::user::UserId;
use crate::database::{Id, MongoCollection};

pub type SubscriptionTimelineId = Id<SubscriptionTimeline>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "subscription_timelines")]
#[mongo(index(fields(product_ids = 1)))]
#[mongo(index(fields("periods.id" = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
pub struct SubscriptionTimeline {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SubscriptionTimelineId,
	/// The ids of the products that this timeline is associated with
	/// for this timeline the periods of time where you had one of these
	/// products are all treated as if they are the same product. Meaning
	/// if you have 2 different products, 1 for 1 month and 1 for 3 months
	/// for all calculations they are treated as if the subscription timeline is
	/// 4 months long.
	pub product_ids: Vec<ProductId>,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub created_by: UserId,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub type SubscriptionTimelinePeriodId = Id<SubscriptionTimelinePeriod>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "subscription_timeline_periods")]
#[mongo(index(fields(timeline_id = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]

pub struct SubscriptionTimelinePeriod {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SubscriptionTimelinePeriodId,
	pub timeline_id: SubscriptionTimelineId,
	pub name: String,
	pub description: Option<String>,
	pub created_by: UserId,
	pub condition: SubscriptionTimelinePeriodCondition,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum SubscriptionTimelinePeriodCondition {
	Duration { duration: DurationUnit },
	TimePeriod { time_period: TimePeriod },
}

pub type UserSubscriptionTimelineId = Id<UserSubscriptionTimeline>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "user_subscription_timelines")]
#[mongo(index(fields(subscription_timeline_id = 1)))]
#[mongo(index(fields(user_id = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct UserSubscriptionTimeline {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: UserSubscriptionTimelineId,
	pub subscription_timeline_id: SubscriptionTimelineId,
	pub user_id: UserId,
	pub periods: Vec<TimePeriod>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[
		MongoGenericCollection::new::<SubscriptionTimeline>(),
		MongoGenericCollection::new::<UserSubscriptionTimeline>(),
	]
}
