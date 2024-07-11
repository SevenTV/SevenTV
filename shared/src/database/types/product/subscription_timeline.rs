use derive_builder::Builder;

use super::{ProductId, TimePeriod};
use crate::database::duration::DurationUnit;
use crate::database::types::GenericCollection;
use crate::database::user::UserId;
use crate::database::{Collection, Id};

pub type SubscriptionTimelineId = Id<SubscriptionTimeline>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
pub struct SubscriptionTimeline {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: SubscriptionTimelineId,
	/// The ids of the products that this timeline is associated with
	/// for this timeline the periods of time where you had one of these
	/// products are all treated as if they are the same product. Meaning
	/// if you have 2 different products, 1 for 1 month and 1 for 3 months
	/// for all calculations they are treated as if the subscription timeline is
	/// 4 months long.
	pub product_ids: Vec<ProductId>,
	pub name: String,
	#[builder(default)]
	pub description: Option<String>,
	pub periods: Vec<SubscriptionTimelinePeriod>,
}

impl Collection for SubscriptionTimeline {
	const COLLECTION_NAME: &'static str = "subscription_timelines";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"product_ids": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"periods.id": 1,
				})
				.options(mongodb::options::IndexOptions::builder().unique(true).build())
				.build(),
		]
	}
}

pub type SubscriptionTimelinePeriodId = Id<SubscriptionTimelinePeriod>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]

pub struct SubscriptionTimelinePeriod {
	pub id: SubscriptionTimelinePeriodId,
	pub name: String,
	pub description: Option<String>,
	pub condition: SubscriptionTimelinePeriodCondition,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum SubscriptionTimelinePeriodCondition {
	Duration { duration: DurationUnit },
	TimePeriod { time_period: TimePeriod },
}

pub type UserSubscriptionTimelineId = Id<UserSubscriptionTimeline>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserSubscriptionTimeline {
	#[serde(rename = "_id")]
	pub id: UserSubscriptionTimelineId,
	pub subscription_timeline_id: SubscriptionTimelineId,
	pub user_id: UserId,
	pub periods: Vec<TimePeriod>,
}

impl Collection for UserSubscriptionTimeline {
	const COLLECTION_NAME: &'static str = "user_subscription_timelines";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"user_id": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"subscription_timeline_id": 1,
				})
				.build(),
		]
	}
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[
		GenericCollection::new::<SubscriptionTimeline>(),
		GenericCollection::new::<UserSubscriptionTimeline>(),
	]
}
