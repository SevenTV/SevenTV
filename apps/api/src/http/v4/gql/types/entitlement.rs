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

use super::{EmoteSet, Paint, Role, SpecialEvent, SubscriptionBenefit, User};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

#[derive(async_graphql::SimpleObject)]
#[graphql(concrete(name = "EntitlementEdgeAnyAny", params(EntitlementNodeAny, EntitlementNodeAny)))]
#[graphql(concrete(name = "EntitlementEdgeAnyPaint", params(EntitlementNodeAny, EntitlementNodePaint)))]
#[graphql(concrete(name = "EntitlementEdgeAnyBadge", params(EntitlementNodeAny, EntitlementNodeBadge)))]
pub struct EntitlementEdge<From: OutputType, To: OutputType> {
	pub from: From,
	pub to: To,
}

#[derive(async_graphql::Union)]
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
			EntitlementEdgeKind::Product { product_id } => Self::Product(EntitlementNodeProduct {
				product_id: product_id.clone(),
			}),
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

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EntitlementNodeUser {
	pub user_id: UserId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodeUser {
	async fn user(&self, ctx: &Context<'_>) -> Result<User, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_loader
			.load(global, self.user_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

		Ok(user.into())
	}
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EntitlementNodeRole {
	pub role_id: RoleId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodeRole {
	async fn role(&self, ctx: &Context<'_>) -> Result<Role, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let role = global
			.role_by_id_loader
			.load(self.role_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load role"))?
			.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "role not found"))?;

		Ok(role.into())
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct EntitlementNodeBadge {
	pub badge_id: BadgeId,
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EntitlementNodePaint {
	pub paint_id: PaintId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodePaint {
	async fn paint(&self, ctx: &Context<'_>) -> Result<Paint, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let paint = global
			.paint_by_id_loader
			.load(self.paint_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load paint"))?
			.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "paint not found"))?;

		Ok(Paint::from_db(paint, &global.config.api.cdn_origin))
	}
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EntitlementNodeEmoteSet {
	pub emote_set_id: EmoteSetId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodeEmoteSet {
	async fn emote_set(&self, ctx: &Context<'_>) -> Result<EmoteSet, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(self.emote_set_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?
			.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "emote set not found"))?;

		Ok(emote_set.into())
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct EntitlementNodeProduct {
	pub product_id: ProductId,
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EntitlementNodeSubscriptionBenefit {
	pub subscription_benefit_id: SubscriptionBenefitId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodeSubscriptionBenefit {
	async fn subscription_benefit(&self, ctx: &Context<'_>) -> Result<SubscriptionBenefit, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		// TODO: Use data loader?
		let product = SubscriptionProduct::collection(&global.db)
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
			.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "subscription benefit not found"))?;

		let subscription_benefit = product
			.benefits
			.into_iter()
			.find(|b| b.id == self.subscription_benefit_id)
			.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "subscription benefit not found"))?;

		Ok(subscription_benefit.into())
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct EntitlementNodeSubscription {
	pub subscription_id: SubscriptionId,
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EntitlementNodeSpecialEvent {
	pub special_event_id: SpecialEventId,
}

#[async_graphql::ComplexObject]
impl EntitlementNodeSpecialEvent {
	async fn special_event(&self, ctx: &Context<'_>) -> Result<SpecialEvent, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		// TODO: Use data loader?
		let special_event = shared::database::product::special_event::SpecialEvent::collection(&global.db)
			.find_one(filter::filter! {
				shared::database::product::special_event::SpecialEvent {
					#[query(rename = "_id")]
					id: self.special_event_id,
				}
			})
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load special event"))?
			.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "special event not found"))?;

		Ok(special_event.into())
	}
}

#[derive(Default, async_graphql::SimpleObject)]
pub struct EntitlementNodeGlobalDefaultEntitlementGroup {
	#[graphql(deprecation = true)]
	pub noop: bool,
}
