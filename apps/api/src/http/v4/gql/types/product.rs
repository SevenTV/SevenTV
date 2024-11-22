use shared::database::product::special_event::SpecialEventId;
use shared::database::product::SubscriptionBenefitId;
use shared::database::user::UserId;

#[derive(async_graphql::SimpleObject)]
pub struct SubscriptionBenefit {
	pub id: SubscriptionBenefitId,
	pub name: String,
	// pub condition: SubscriptionBenefitCondition,
}

impl From<shared::database::product::SubscriptionBenefit> for SubscriptionBenefit {
	fn from(benefit: shared::database::product::SubscriptionBenefit) -> Self {
		Self {
			id: benefit.id,
			name: benefit.name,
			// condition: benefit.condition.into(),
		}
	}
}

// #[derive(async_graphql::SimpleObject)]
// pub struct SubscriptionBenefitCondition {

// }

#[derive(async_graphql::SimpleObject)]
pub struct SpecialEvent {
	pub id: SpecialEventId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub created_by_id: UserId,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<shared::database::product::special_event::SpecialEvent> for SpecialEvent {
	fn from(event: shared::database::product::special_event::SpecialEvent) -> Self {
		Self {
			id: event.id,
			name: event.name,
			description: event.description,
			tags: event.tags,
			created_by_id: event.created_by,
			updated_at: event.updated_at,
			search_updated_at: event.search_updated_at,
		}
	}
}
