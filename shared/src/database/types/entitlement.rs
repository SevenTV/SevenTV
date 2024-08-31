use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::badge::BadgeId;
use super::emote_set::EmoteSetId;
use super::paint::PaintId;
use super::product::codes::RedeemCodeId;
use super::product::{InvoiceId, InvoiceLineItemId, ProductId, SubscriptionId};
use super::role::RoleId;
use super::user::UserId;
use super::{MongoCollection, MongoGenericCollection};
use crate::database::graph::{GraphEdge, GraphKey};
use crate::database::Id;
use crate::typesense::types::impl_typesense_type;

/// https://www.mermaidchart.com/raw/db698878-667d-4aac-a7c7-6c310120ff35?version=v0.1&format=svg
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EntitlementEdgeKind {
	User { user_id: UserId },
	Role { role_id: RoleId },
	Badge { badge_id: BadgeId },
	Paint { paint_id: PaintId },
	EmoteSet { emote_id: EmoteSetId },
	Product { product_id: ProductId },
	SubscriptionProduct { product_id: ProductId },
	Subscription { subscription_id: SubscriptionId },
	EntitlementGroup { entitlement_group_id: EntitlementGroupId },
	GlobalDefaultEntitlementGroup,
}

impl std::fmt::Display for EntitlementEdgeKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			EntitlementEdgeKind::User { user_id } => write!(f, "user:{}", user_id),
			EntitlementEdgeKind::Role { role_id } => write!(f, "role:{}", role_id),
			EntitlementEdgeKind::Badge { badge_id } => write!(f, "badge:{}", badge_id),
			EntitlementEdgeKind::Paint { paint_id } => write!(f, "paint:{}", paint_id),
			EntitlementEdgeKind::EmoteSet { emote_id } => write!(f, "emote_set:{}", emote_id),
			EntitlementEdgeKind::Product { product_id } => write!(f, "product:{}", product_id),
			EntitlementEdgeKind::SubscriptionProduct { product_id } => write!(f, "subscription_product:{}", product_id),
			EntitlementEdgeKind::Subscription { subscription_id } => write!(f, "subscription:{}", subscription_id),
			EntitlementEdgeKind::EntitlementGroup { entitlement_group_id } => {
				write!(f, "entitlement_group:{}", entitlement_group_id)
			}
			EntitlementEdgeKind::GlobalDefaultEntitlementGroup => write!(f, "global_default_entitlement_group"),
		}
	}
}

impl std::str::FromStr for EntitlementEdgeKind {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let parts: Vec<&str> = s.splitn(2, ':').collect();
		if parts.len() < 2 {
			return Err("invalid format");
		}

		let kind = match parts[0] {
			"user" => EntitlementEdgeKind::User {
				user_id: parts[1].parse().map_err(|_| "invalid user id")?,
			},
			"role" => EntitlementEdgeKind::Role {
				role_id: parts[1].parse().map_err(|_| "invalid role id")?,
			},
			"badge" => EntitlementEdgeKind::Badge {
				badge_id: parts[1].parse().map_err(|_| "invalid badge id")?,
			},
			"paint" => EntitlementEdgeKind::Paint {
				paint_id: parts[1].parse().map_err(|_| "invalid paint id")?,
			},
			"emote_set" => EntitlementEdgeKind::EmoteSet {
				emote_id: parts[1].parse().map_err(|_| "invalid emote set id")?,
			},
			"product" => EntitlementEdgeKind::Product {
				product_id: parts[1].parse().map_err(|_| "invalid product id")?,
			},
			"subscription" => EntitlementEdgeKind::Subscription {
				subscription_id: parts[1].parse().map_err(|_| "invalid subscription id")?,
			},
			"entitlement_group" => EntitlementEdgeKind::EntitlementGroup {
				entitlement_group_id: parts[1].parse().map_err(|_| "invalid entitlement group id")?,
			},
			"global_default_entitlement_group" => EntitlementEdgeKind::GlobalDefaultEntitlementGroup,
			_ => return Err("invalid kind"),
		};

		Ok(kind)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntitlementEdgeKindString(pub EntitlementEdgeKind);

impl From<EntitlementEdgeKindString> for EntitlementEdgeKind {
	fn from(s: EntitlementEdgeKindString) -> Self {
		s.0
	}
}

impl From<EntitlementEdgeKind> for EntitlementEdgeKindString {
	fn from(k: EntitlementEdgeKind) -> Self {
		Self(k)
	}
}

impl std::fmt::Display for EntitlementEdgeKindString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl Serialize for EntitlementEdgeKindString {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.to_string().serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for EntitlementEdgeKindString {
	fn deserialize<D>(deserializer: D) -> Result<EntitlementEdgeKindString, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		Ok(EntitlementEdgeKindString(s.parse().map_err(serde::de::Error::custom)?))
	}
}

impl_typesense_type!(EntitlementEdgeKindString, String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntitlementEdgeManagedBy {
	InvoiceLineItem {
		invoice_id: InvoiceId,
		line_item_id: InvoiceLineItemId,
	},
	RedeemCode {
		redeem_code_id: RedeemCodeId,
	},
}

impl std::fmt::Display for EntitlementEdgeManagedBy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::RedeemCode { redeem_code_id } => write!(f, "redeem_code:{redeem_code_id}"),
			Self::InvoiceLineItem {
				invoice_id,
				line_item_id,
			} => write!(f, "invoice:{invoice_id}:{line_item_id}"),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct EntitlementEdgeId {
	pub from: EntitlementEdgeKind,
	pub to: EntitlementEdgeKind,
	pub managed_by: Option<EntitlementEdgeManagedBy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, MongoCollection)]
#[mongo(collection_name = "entitlement_edges")]
#[mongo(index(fields("_id.from" = 1, "_id.to" = 1)))]
#[mongo(index(fields("_id.to" = 1, "_id.from" = 1)))]
#[mongo(index(fields("_id.managed_by" = 1)))]
#[serde(deny_unknown_fields)]
pub struct EntitlementEdge {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: EntitlementEdgeId,
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
			Self::Product { .. }
				| Self::Badge { .. }
				| Self::Paint { .. }
				| Self::EmoteSet { .. }
				| Self::Role { .. }
				| Self::EntitlementGroup { .. }
				| Self::Subscription { .. }
		)
	}

	fn has_outbound(&self) -> bool {
		matches!(
			self,
			Self::User { .. }
				| Self::Role { .. }
				| Self::Product { .. }
				| Self::Subscription { .. }
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

#[derive(Debug, Clone, Serialize, Deserialize, MongoCollection)]
#[mongo(collection_name = "entitlement_groups")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct EntitlementGroup {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: EntitlementGroupId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[
		MongoGenericCollection::new::<EntitlementEdge>(),
		MongoGenericCollection::new::<EntitlementGroup>(),
	]
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CalculatedEntitlements {
	pub roles: HashSet<RoleId>,
	pub badges: HashSet<BadgeId>,
	pub paints: HashSet<PaintId>,
	pub emote_sets: HashSet<EmoteSetId>,
	pub products: HashSet<ProductId>,
	pub subscriptions: HashSet<SubscriptionId>,
	pub entitlement_groups: HashSet<EntitlementGroupId>,
}

impl CalculatedEntitlements {
	pub fn new(edges: impl IntoIterator<Item = EntitlementEdgeKind>) -> Self {
		let mut roles = HashSet::new();
		let mut badges = HashSet::new();
		let mut paints = HashSet::new();
		let mut emote_sets = HashSet::new();
		let mut products = HashSet::new();
		let mut subscriptions = HashSet::new();
		let mut entitlement_groups = HashSet::new();

		edges.into_iter().for_each(|to| match to {
			EntitlementEdgeKind::Role { role_id } => {
				roles.insert(role_id);
			}
			EntitlementEdgeKind::Badge { badge_id } => {
				badges.insert(badge_id);
			}
			EntitlementEdgeKind::Paint { paint_id } => {
				paints.insert(paint_id);
			}
			EntitlementEdgeKind::EmoteSet { emote_id } => {
				emote_sets.insert(emote_id);
			}
			EntitlementEdgeKind::Product { product_id } => {
				products.insert(product_id);
			}
			EntitlementEdgeKind::SubscriptionProduct { product_id } => {
				products.insert(product_id);
			}
			EntitlementEdgeKind::Subscription { subscription_id } => {
				subscriptions.insert(subscription_id);
			}
			EntitlementEdgeKind::EntitlementGroup { entitlement_group_id } => {
				entitlement_groups.insert(entitlement_group_id);
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
			subscriptions,
			entitlement_groups,
		}
	}
}
