use chrono::Utc;

use crate::database;
use crate::database::entitlement::{EntitlementEdgeKind, EntitlementEdgeKindString};
use crate::database::product::subscription_timeline::{SubscriptionTimelineId, SubscriptionTimelinePeriodId};
use crate::database::product::ProductId;
use crate::database::user::UserId;
use crate::typesense::types::duration_unit::DurationUnit;
use crate::typesense::types::{impl_typesense_type, TypesenseCollection, TypesenseGenericCollection};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "subscription_timelines")]
#[serde(deny_unknown_fields)]
pub struct SubscriptionTimeline {
	pub id: SubscriptionTimelineId,
	pub product_ids: Vec<ProductId>,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub created_by: UserId,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::product::subscription_timeline::SubscriptionTimeline> for SubscriptionTimeline {
	fn from(value: database::product::subscription_timeline::SubscriptionTimeline) -> Self {
		Self {
			id: value.id,
			product_ids: value.product_ids,
			name: value.name,
			description: value.description,
			tags: value.tags,
			created_by: value.created_by,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "subscription_timeline_periods")]
#[serde(deny_unknown_fields)]
pub struct SubscriptionTimelinePeriod {
	pub id: SubscriptionTimelinePeriodId,
	pub timeline_id: SubscriptionTimelineId,
	pub name: String,
	pub description: Option<String>,
	pub created_by: UserId,
	pub condition_kind: SubscriptionTimelinePeriodConditionKind,
	pub condition_start_at: Option<i64>,
	pub condition_end_at: Option<i64>,
	pub condition_length_kind: Option<DurationUnit>,
	pub condition_length_value: Option<i32>,
	pub entitlement_grants: Vec<EntitlementEdgeKindString>,
	pub created_at: i64,
	pub updated_at: i64,
}

impl SubscriptionTimelinePeriod {
	pub fn from_db(
		value: database::product::subscription_timeline::SubscriptionTimelinePeriod,
		entitlement_grants: impl IntoIterator<Item = EntitlementEdgeKind>,
	) -> Self {
		Self {
			id: value.id,
			timeline_id: value.timeline_id,
			name: value.name,
			description: value.description,
			created_by: value.created_by,
			condition_kind: match value.condition {
				database::product::subscription_timeline::SubscriptionTimelinePeriodCondition::Duration { .. } => {
					SubscriptionTimelinePeriodConditionKind::Duration
				}
				database::product::subscription_timeline::SubscriptionTimelinePeriodCondition::TimePeriod { .. } => {
					SubscriptionTimelinePeriodConditionKind::TimePeriod
				}
			},
			condition_start_at: match &value.condition {
				database::product::subscription_timeline::SubscriptionTimelinePeriodCondition::TimePeriod {
					time_period,
				} => Some(time_period.start.timestamp_millis()),
				database::product::subscription_timeline::SubscriptionTimelinePeriodCondition::Duration { .. } => None,
			},
			condition_end_at: match &value.condition {
				database::product::subscription_timeline::SubscriptionTimelinePeriodCondition::TimePeriod {
					time_period,
				} => Some(time_period.end.timestamp_millis()),
				database::product::subscription_timeline::SubscriptionTimelinePeriodCondition::Duration { .. } => None,
			},
			condition_length_kind: match &value.condition {
				database::product::subscription_timeline::SubscriptionTimelinePeriodCondition::Duration { duration } => {
					Some(match duration {
						database::duration::DurationUnit::Days(_) => DurationUnit::Days,
						database::duration::DurationUnit::Months(_) => DurationUnit::Months,
					})
				}
				database::product::subscription_timeline::SubscriptionTimelinePeriodCondition::TimePeriod { .. } => None,
			},
			condition_length_value: match value.condition {
				database::product::subscription_timeline::SubscriptionTimelinePeriodCondition::Duration { duration } => {
					Some(match duration {
						database::duration::DurationUnit::Days(x) => x,
						database::duration::DurationUnit::Months(x) => x,
					})
				}
				database::product::subscription_timeline::SubscriptionTimelinePeriodCondition::TimePeriod { .. } => None,
			},
			entitlement_grants: entitlement_grants.into_iter().map(Into::into).collect(),
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
		}
	}
}

#[derive(Debug, Clone, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(i32)]
pub enum SubscriptionTimelinePeriodConditionKind {
	Duration = 0,
	TimePeriod = 1,
}

impl_typesense_type!(SubscriptionTimelinePeriod, Object);
impl_typesense_type!(SubscriptionTimelinePeriodConditionKind, Int32);

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[
		TypesenseGenericCollection::new::<SubscriptionTimeline>(),
		TypesenseGenericCollection::new::<SubscriptionTimelinePeriod>(),
	]
}
