use shared::database::product::SubscriptionBenefitId;

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
