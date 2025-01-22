use itertools::Itertools;
use shared::database::Id;
use shared::database::{entitlement::EntitlementEdgeKind, product::subscription::SubscriptionId};
use std::str::FromStr;

use super::{EntitlementEdge, EntitlementNodeAny};

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct RawEntitlements {
	pub edges: Vec<EntitlementEdge<EntitlementNodeAny, EntitlementNodeAny>>,
}

impl RawEntitlements {
	pub fn from_db(edges: &[shared::database::entitlement::EntitlementEdge]) -> Self {
		Self {
			edges: edges.iter().unique().map(EntitlementEdge::from_db).collect(),
		}
	}
}

#[async_graphql::ComplexObject]
impl RawEntitlements {
	async fn nodes(&self) -> Vec<&EntitlementNodeAny> {
		self.edges.iter().flat_map(|edge| [&edge.from, &edge.to]).unique().collect()
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, async_graphql::Enum)]
pub enum EntitlementNodeTypeInput {
	User,
	Role,
	Badge,
	Paint,
	EmoteSet,
	SubscriptionBenefit,
	SpecialEvent,
	GlobalDefaultEntitlementGroup,
	Subscription,
}

#[derive(async_graphql::InputObject)]
pub struct EntitlementNodeInput {
	#[graphql(name = "type")]
	ty: EntitlementNodeTypeInput,
	id: Id<()>,
}

impl From<EntitlementNodeInput> for EntitlementEdgeKind {
	fn from(value: EntitlementNodeInput) -> Self {
		match value.ty {
			EntitlementNodeTypeInput::User => EntitlementEdgeKind::User {
				user_id: value.id.cast(),
			},
			EntitlementNodeTypeInput::Role => EntitlementEdgeKind::Role {
				role_id: value.id.cast(),
			},
			EntitlementNodeTypeInput::Badge => EntitlementEdgeKind::Badge {
				badge_id: value.id.cast(),
			},
			EntitlementNodeTypeInput::Paint => EntitlementEdgeKind::Paint {
				paint_id: value.id.cast(),
			},
			EntitlementNodeTypeInput::EmoteSet => EntitlementEdgeKind::EmoteSet {
				emote_set_id: value.id.cast(),
			},
			EntitlementNodeTypeInput::SubscriptionBenefit => EntitlementEdgeKind::SubscriptionBenefit {
				subscription_benefit_id: value.id.cast(),
			},
			EntitlementNodeTypeInput::SpecialEvent => EntitlementEdgeKind::SpecialEvent {
				special_event_id: value.id.cast(),
			},
			EntitlementNodeTypeInput::GlobalDefaultEntitlementGroup => EntitlementEdgeKind::GlobalDefaultEntitlementGroup,
			EntitlementNodeTypeInput::Subscription => EntitlementEdgeKind::Subscription {
				subscription_id: SubscriptionId {
					user_id: value.id.cast(),
					// Use hardcoded product_id for now.
					product_id: Id::from_str("01FEVKBBTGRAT7FCY276TNTJ4A").unwrap(),
				},
			},
		}
	}
}
