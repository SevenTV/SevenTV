use std::sync::Arc;

use async_graphql::{Context, OutputType};
use shared::database::badge::BadgeId;
use shared::database::emote_set::EmoteSetId;
use shared::database::entitlement::EntitlementEdgeKind;
use shared::database::paint::PaintId;
use shared::database::product::special_event::SpecialEventId;
use shared::database::product::subscription::SubscriptionId;
use shared::database::product::{ProductId, SubscriptionBenefitId, SubscriptionProduct};
use shared::database::queries::filter;
use shared::database::role::RoleId;
use shared::database::user::UserId;
use shared::database::MongoCollection;

use super::{Badge, EmoteSet, Paint, Product, Role, SpecialEvent, SubscriptionBenefit, User};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

pub mod raw_entitlement;

#[allow(clippy::duplicated_attributes)]
#[derive(async_graphql::SimpleObject)]
#[graphql(concrete(name = "EntitlementEdgeAnyAny", params(EntitlementNodeAny, EntitlementNodeAny)))]
#[graphql(concrete(name = "EntitlementEdgeAnyPaint", params(EntitlementNodeAny, EntitlementNodePaint)))]
#[graphql(concrete(name = "EntitlementEdgeAnyBadge", params(EntitlementNodeAny, EntitlementNodeBadge)))]
#[graphql(concrete(name = "EntitlementEdgeAnyProduct", params(EntitlementNodeAny, EntitlementNodeProduct)))]
pub struct EntitlementEdge<From: OutputType, To: OutputType> {
	pub from: From,
	pub to: To,
}

impl EntitlementEdge<EntitlementNodeAny, EntitlementNodeAny> {
	pub fn from_db(edge: &shared::database::entitlement::EntitlementEdge) -> Self {
		Self {
			from: EntitlementNodeAny::from_db(&edge.id.from),
			to: EntitlementNodeAny::from_db(&edge.id.to),
		}
	}
}

#[derive(async_graphql::Union, Clone, PartialEq, Eq, Hash)]
pub enum EntitlementNodeAny {
	User(EntitlementNodeUser),
	Role(EntitlementNodeRole),
	Badge(EntitlementNodeBadge),
	Paint(EntitlementNodePaint),
	EmoteSet(EntitlementNodeEmoteSet),
	Product(EntitlementNodeProduct),
	SubscriptionBenefit(EntitlementNodeSubscriptionBenefit),
	Subscription(EntitlementNodeSubscription),
	SpecialEvent(EntitlementNodeSpecialEvent),
	GlobalDefaultEntitlementGroup(EntitlementNodeGlobalDefaultEntitlementGroup),
}

impl EntitlementNodeAny {
	pub fn from_db(value: &EntitlementEdgeKind) -> Self {
		match value {
			EntitlementEdgeKind::User { user_id } => Self::User(EntitlementNodeUser { user_id: *user_id }),
			EntitlementEdgeKind::Role { role_id } => Self::Role(EntitlementNodeRole { role_id: *role_id }),
			EntitlementEdgeKind::Badge { badge_id } => Self::Badge(EntitlementNodeBadge { badge_id: *badge_id }),
			EntitlementEdgeKind::Paint { paint_id } => Self::Paint(EntitlementNodePaint { paint_id: *paint_id }),
			EntitlementEdgeKind::EmoteSet { emote_set_id } => Self::EmoteSet(EntitlementNodeEmoteSet {
				emote_set_id: *emote_set_id,
			}),
			EntitlementEdgeKind::Product { product_id } => Self::Product(EntitlementNodeProduct { product_id: *product_id }),
			EntitlementEdgeKind::SubscriptionBenefit { subscription_benefit_id } => {
				Self::SubscriptionBenefit(EntitlementNodeSubscriptionBenefit {
					subscription_benefit_id: *subscription_benefit_id,
				})
			}
			EntitlementEdgeKind::Subscription { subscription_id } => Self::Subscription(EntitlementNodeSubscription {
				subscription_id: *subscription_id,
			}),
			EntitlementEdgeKind::SpecialEvent { special_event_id } => Self::SpecialEvent(EntitlementNodeSpecialEvent {
				special_event_id: *special_event_id,
			}),
			EntitlementEdgeKind::GlobalDefaultEntitlementGroup => {
				Self::GlobalDefaultEntitlementGroup(EntitlementNodeGlobalDefaultEntitlementGroup::default())
			}
		}
	}
}

#[derive(async_graphql::SimpleObject, Clone, PartialEq, Eq, Hash)]
#[graphql(complex)]
pub struct EntitlementNodeUser {
	pub user_id: UserId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodeUser {
	#[tracing::instrument(skip_all, name = "EntitlementNodeUser::user")]
	async fn user(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_loader
			.load(global, self.user_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(user.map(Into::into))
	}
}

#[derive(async_graphql::SimpleObject, Clone, PartialEq, Eq, Hash)]
#[graphql(complex)]
pub struct EntitlementNodeRole {
	pub role_id: RoleId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodeRole {
	#[tracing::instrument(skip_all, name = "EntitlementNodeRole::role")]
	async fn role(&self, ctx: &Context<'_>) -> Result<Option<Role>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let role = global
			.role_by_id_loader
			.load(self.role_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load role"))?;

		Ok(role.map(Into::into))
	}
}

#[derive(async_graphql::SimpleObject, Clone, PartialEq, Eq, Hash)]
#[graphql(complex)]
pub struct EntitlementNodeBadge {
	pub badge_id: BadgeId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodeBadge {
	#[tracing::instrument(skip_all, name = "EntitlementNodeBadge::badge")]
	async fn badge(&self, ctx: &Context<'_>) -> Result<Option<Badge>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let badge = global
			.badge_by_id_loader
			.load(self.badge_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load badge"))?;

		Ok(badge.map(|b| Badge::from_db(b, &global.config.api.cdn_origin)))
	}
}

#[derive(async_graphql::SimpleObject, Clone, PartialEq, Eq, Hash)]
#[graphql(complex)]
pub struct EntitlementNodePaint {
	pub paint_id: PaintId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodePaint {
	#[tracing::instrument(skip_all, name = "EntitlementNodePaint::paint")]
	async fn paint(&self, ctx: &Context<'_>) -> Result<Option<Paint>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let paint = global
			.paint_by_id_loader
			.load(self.paint_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load paint"))?;

		Ok(paint.map(|p| Paint::from_db(p, &global.config.api.cdn_origin)))
	}
}

#[derive(async_graphql::SimpleObject, Clone, PartialEq, Eq, Hash)]
#[graphql(complex)]
pub struct EntitlementNodeEmoteSet {
	pub emote_set_id: EmoteSetId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodeEmoteSet {
	#[tracing::instrument(skip_all, name = "EntitlementNodeEmoteSet::emote_set")]
	async fn emote_set(&self, ctx: &Context<'_>) -> Result<Option<EmoteSet>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(self.emote_set_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?;

		Ok(emote_set.map(Into::into))
	}
}

#[derive(async_graphql::SimpleObject, Clone, PartialEq, Eq, Hash)]
pub struct EntitlementNodeProduct {
	pub product_id: ProductId,
}
#[async_graphql::ComplexObject]
impl EntitlementNodeProduct {
	#[tracing::instrument(skip_all, name = "EntitlementNodeBadge::badge")]
	async fn product(&self, ctx: &Context<'_>) -> Result<Option<Product>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let product = global
			.product_by_id_loader
			.load(self.product_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load product"))?;

		Ok(product.map(Into::into))
	}
}

#[derive(async_graphql::SimpleObject, Clone, PartialEq, Eq, Hash)]
#[graphql(complex)]
pub struct EntitlementNodeSubscriptionBenefit {
	pub subscription_benefit_id: SubscriptionBenefitId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodeSubscriptionBenefit {
	#[tracing::instrument(skip_all, name = "EntitlementNodeSubscriptionBenefit::subscription_benefit")]
	async fn subscription_benefit(&self, ctx: &Context<'_>) -> Result<Option<SubscriptionBenefit>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		// TODO: Use data loader?
		let Some(product) = SubscriptionProduct::collection(&global.db)
			.find_one(filter::filter! {
				SubscriptionProduct {
					#[query(flatten)]
					benefits: shared::database::product::SubscriptionBenefit {
						id: self.subscription_benefit_id,
					}
				}
			})
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription benefit"))?
		else {
			return Ok(None);
		};

		let subscription_benefit = product.benefits.into_iter().find(|b| b.id == self.subscription_benefit_id);

		Ok(subscription_benefit.map(Into::into))
	}
}

#[derive(async_graphql::SimpleObject, Clone, PartialEq, Eq, Hash)]
pub struct EntitlementNodeSubscription {
	pub subscription_id: SubscriptionId,
}

#[derive(async_graphql::SimpleObject, Clone, PartialEq, Eq, Hash)]
#[graphql(complex)]
pub struct EntitlementNodeSpecialEvent {
	pub special_event_id: SpecialEventId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodeSpecialEvent {
	#[tracing::instrument(skip_all, name = "EntitlementNodeSpecialEvent::special_event")]
	async fn special_event(&self, ctx: &Context<'_>) -> Result<Option<SpecialEvent>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let special_event = global
			.special_event_by_id_loader
			.load(self.special_event_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load special event"))?;

		Ok(special_event.map(Into::into))
	}
}

#[derive(Default, async_graphql::SimpleObject, Clone, PartialEq, Eq, Hash)]
pub struct EntitlementNodeGlobalDefaultEntitlementGroup {
	#[graphql(deprecation = true)]
	pub noop: bool,
}

#[derive(Default, async_graphql::SimpleObject, Clone, PartialEq, Eq, Hash)]
pub struct EntitlementNodeAllCosmetics {
	#[graphql(deprecation = true)]
	pub noop: bool,
}
