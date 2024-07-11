use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use derive_builder::Builder;

use super::badge::BadgeId;
use super::emote_set::EmoteSetId;
use super::paint::PaintId;
use super::product::codes::{GiftCodeId, RedeemCodeId};
use super::product::promotion::PromotionId;
use super::product::subscription_timeline::{
	SubscriptionTimelineId, SubscriptionTimelinePeriodId, UserSubscriptionTimelineId,
};
use super::product::{InvoiceId, InvoiceLineItemId, ProductId};
use super::role::RoleId;
use super::user::{EntitlementCacheKey, UserId, UserSearchIndex};
use super::{Collection, GenericCollection};
use crate::database::graph::{GraphEdge, GraphKey};
use crate::database::Id;

/// https://www.mermaidchart.com/raw/db698878-667d-4aac-a7c7-6c310120ff35?version=v0.1&format=svg
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EntitlementEdgeKind {
	User {
		user_id: UserId,
	},
	Role {
		role_id: RoleId,
	},
	Badge {
		badge_id: BadgeId,
	},
	Paint {
		paint_id: PaintId,
	},
	EmoteSet {
		emote_id: EmoteSetId,
	},
	Product {
		product_id: ProductId,
	},
	SubscriptionTimelinePeriod {
		subscription_timeline_id: SubscriptionTimelineId,
		period_id: SubscriptionTimelinePeriodId,
	},
	Promotion {
		promotion_id: PromotionId,
	},
	UserSubscriptionTimeline {
		user_subscription_timeline_id: UserSubscriptionTimelineId,
	},
	EntitlementGroup {
		entitlement_group_id: Id<EntitlementGroup>,
	},
	GlobalDefaultEntitlementGroup,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntitlementEdgeManagedBy {
	InvoiceLineItem {
		invoice_id: InvoiceId,
		line_item_id: InvoiceLineItemId,
	},
	GiftCode {
		gift_id: GiftCodeId,
	},
	RedeemCode {
		redeem_code_id: RedeemCodeId,
	},
	Promotion {
		promotion: PromotionId,
	},
	UserSubscriptionTimeline {
		user_subscription_timeline_id: UserSubscriptionTimelineId,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, Builder)]
pub struct EntitlementEdgeId {
	pub from: EntitlementEdgeKind,
	pub to: EntitlementEdgeKind,
	#[builder(default)]
	pub managed_by: Option<EntitlementEdgeManagedBy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, Builder)]
pub struct EntitlementEdge {
	#[serde(rename = "_id")]
	pub id: EntitlementEdgeId,
}

impl Collection for EntitlementEdge {
	const COLLECTION_NAME: &'static str = "entitlement_edges";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"_id.from": 1,
					"_id.to": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"_id.to": 1,
					"_id.from": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"_id.managed_by": 1,
				})
				.build(),
		]
	}
}

impl GraphEdge for EntitlementEdge {
	type Key = EntitlementEdgeKind;

	fn edge_next(&self, direction: crate::database::graph::Direction) -> impl IntoIterator<Item = Self::Key> {
		match direction {
			crate::database::graph::Direction::Inbound => std::iter::once(self.id.from.clone()),
			crate::database::graph::Direction::Outbound => std::iter::once(self.id.to.clone()),
		}
	}
}

impl GraphKey for EntitlementEdgeKind {
	fn has_inbound(&self) -> bool {
		matches!(
			self,
			Self::UserSubscriptionTimeline { .. }
				| Self::Promotion { .. }
				| Self::Product { .. }
				| Self::SubscriptionTimelinePeriod { .. }
				| Self::Badge { .. }
				| Self::Paint { .. }
				| Self::EmoteSet { .. }
				| Self::Role { .. }
				| Self::EntitlementGroup { .. }
		)
	}

	fn has_outbound(&self) -> bool {
		matches!(
			self,
			Self::User { .. }
				| Self::Role { .. }
				| Self::Product { .. }
				| Self::Promotion { .. }
				| Self::SubscriptionTimelinePeriod { .. }
				| Self::UserSubscriptionTimeline { .. }
				| Self::EntitlementGroup { .. }
				| Self::GlobalDefaultEntitlementGroup
		)
	}
}

impl EntitlementEdge {
	pub fn new(from: EntitlementEdgeKind, to: EntitlementEdgeKind, managed_by: Option<EntitlementEdgeManagedBy>) -> Self {
		Self {
			id: EntitlementEdgeId { from, to, managed_by },
		}
	}
}

pub type EntitlementGroupId = Id<EntitlementGroup>;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct EntitlementGroup {
	#[builder(default)]
	pub id: EntitlementGroupId,
	pub name: String,
	#[builder(default)]
	pub description: Option<String>,
}

impl Collection for EntitlementGroup {
	const COLLECTION_NAME: &'static str = "entitlement_groups";
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[
		GenericCollection::new::<EntitlementEdge>(),
		GenericCollection::new::<EntitlementGroup>(),
	]
}

#[derive(Debug, Clone, Default)]
pub struct CalculatedEntitlements {
	pub roles: HashSet<RoleId>,
	pub badges: HashSet<BadgeId>,
	pub paints: HashSet<PaintId>,
	pub emote_sets: HashSet<EmoteSetId>,
	pub products: HashSet<ProductId>,
	pub promotions: HashSet<PromotionId>,
	pub user_subscription_timelines: HashSet<UserSubscriptionTimelineId>,
	pub subscription_timelines: HashSet<(SubscriptionTimelineId, SubscriptionTimelinePeriodId)>,
	pub entitlement_groups: HashSet<EntitlementGroupId>,
}

impl CalculatedEntitlements {
	pub fn new(edges: &[EntitlementEdge]) -> Self {
		let mut roles = HashSet::new();
		let mut badges = HashSet::new();
		let mut paints = HashSet::new();
		let mut emote_sets = HashSet::new();
		let mut products = HashSet::new();
		let mut subscription_timelines = HashSet::new();
		let mut promotions = HashSet::new();
		let mut user_subscription_timelines = HashSet::new();
		let mut entitlement_groups = HashSet::new();

		edges.iter().for_each(|edge| match &edge.id.to {
			EntitlementEdgeKind::Role { role_id } => {
				roles.insert(role_id.clone());
			}
			EntitlementEdgeKind::Badge { badge_id } => {
				badges.insert(badge_id.clone());
			}
			EntitlementEdgeKind::Paint { paint_id } => {
				paints.insert(paint_id.clone());
			}
			EntitlementEdgeKind::EmoteSet { emote_id } => {
				emote_sets.insert(emote_id.clone());
			}
			EntitlementEdgeKind::Product { product_id } => {
				products.insert(product_id.clone());
			}
			EntitlementEdgeKind::SubscriptionTimelinePeriod {
				subscription_timeline_id,
				period_id,
			} => {
				subscription_timelines.insert((subscription_timeline_id.clone(), period_id.clone()));
			}
			EntitlementEdgeKind::Promotion { promotion_id } => {
				promotions.insert(promotion_id.clone());
			}
			EntitlementEdgeKind::UserSubscriptionTimeline {
				user_subscription_timeline_id,
			} => {
				user_subscription_timelines.insert(user_subscription_timeline_id.clone());
			}
			EntitlementEdgeKind::EntitlementGroup { entitlement_group_id } => {
				entitlement_groups.insert(entitlement_group_id.clone());
			}
			EntitlementEdgeKind::User { .. } => {
				tracing::warn!("user entitlements are not supported in this context")
			}
			EntitlementEdgeKind::GlobalDefaultEntitlementGroup => {}
		});

		Self {
			roles,
			badges,
			paints,
			emote_sets,
			products,
			subscription_timelines,
			promotions,
			user_subscription_timelines,
			entitlement_groups,
		}
	}

	pub fn new_from_cache(cache: &UserSearchIndex) -> Self {
		let mut roles = HashSet::new();
		let badges = HashSet::from_iter(cache.entitled_badges.iter().cloned());
		let paints = HashSet::from_iter(cache.entitled_paints.iter().cloned());
		let emote_sets = HashSet::from_iter(cache.entitled_emote_set_ids.iter().cloned());
		let mut products = HashSet::new();
		let mut subscription_timelines = HashSet::new();
		let mut promotions = HashSet::new();
		let mut user_subscription_timelines = HashSet::new();
		let mut entitlement_groups = HashSet::new();

		cache.entitlement_cache_keys.iter().for_each(|key| match key {
			EntitlementCacheKey::Role { role_id } => {
				roles.insert(role_id.clone());
			}
			EntitlementCacheKey::Product { product_id } => {
				products.insert(product_id.clone());
			}
			EntitlementCacheKey::Promotion { promotion_id } => {
				promotions.insert(promotion_id.clone());
			}
			EntitlementCacheKey::SubscriptionTimeline { .. } => {}
			EntitlementCacheKey::SubscriptionTimelinePeriod {
				subscription_timeline_id,
				period_id,
			} => {
				subscription_timelines.insert((subscription_timeline_id.clone(), period_id.clone()));
			}
			EntitlementCacheKey::UserSubscriptionTimeline {
				user_subscription_timeline_id,
			} => {
				user_subscription_timelines.insert(user_subscription_timeline_id.clone());
			}
			EntitlementCacheKey::EntitlementGroup { entitlement_group_id } => {
				entitlement_groups.insert(entitlement_group_id.clone());
			}
		});

		Self {
			roles,
			badges,
			paints,
			emote_sets,
			products,
			subscription_timelines,
			promotions,
			user_subscription_timelines,
			entitlement_groups,
		}
	}
}
