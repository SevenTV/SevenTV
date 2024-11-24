use std::sync::Arc;

use async_graphql::Context;
use shared::database::badge::BadgeId;
use shared::database::entitlement::EntitlementEdgeKind;
use shared::database::entitlement_edge::EntitlementEdgeGraphTraverse;
use shared::database::graph::{Direction, GraphTraverse};
use shared::database::paint::PaintId;
use shared::database::product::{SubscriptionBenefitCondition, SubscriptionProduct};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::{Badge, Paint};
use crate::sub_refresh_job;

#[derive(Default)]
pub struct StoreQuery;

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct BadgeProgress {
	pub current_badge_id: Option<BadgeId>,
	pub next_badge: Option<BadgeProgressNextBadge>,
}

#[async_graphql::ComplexObject]
impl BadgeProgress {
	async fn current_badge(&self, ctx: &Context<'_>) -> Result<Option<Badge>, ApiError> {
		let Some(current_badge_id) = self.current_badge_id else {
			return Ok(None);
		};

		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let badge = global
			.badge_by_id_loader
			.load(current_badge_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load badge"))?
			.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "badge not found"))?;

		Ok(Some(Badge::from_db(badge, &global.config.api.cdn_origin)))
	}
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct BadgeProgressNextBadge {
	pub badge_id: BadgeId,
	pub percentage: f64,
	pub days_left: f64,
}

#[async_graphql::ComplexObject]
impl BadgeProgressNextBadge {
	async fn badge(&self, ctx: &Context<'_>) -> Result<Badge, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let badge = global
			.badge_by_id_loader
			.load(self.badge_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load badge"))?
			.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "badge not found"))?;

		Ok(Badge::from_db(badge, &global.config.api.cdn_origin))
	}
}

#[async_graphql::Object]
impl StoreQuery {
	async fn badge_progress(&self, ctx: &Context<'_>) -> Result<BadgeProgress, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;

		let user_id = session
			.user_id()
			.ok_or_else(|| ApiError::unauthorized(ApiErrorCode::LoginRequired, "login required"))?;

		// let periods: Vec<_> = global
		// 	.subscription_periods_by_user_id_loader
		// 	.load(user_id)
		// 	.await
		// 	.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError,
		// "failed to load subscription periods"))? 	.unwrap_or_default();

		// let Some(active_period) = periods
		// 	.iter()
		// 	.find(|p| p.start < chrono::Utc::now() && p.end > chrono::Utc::now())
		// 	.cloned()
		// else {
		// 	return Ok(None);
		// };

		// let subscription = global
		// 	.subscription_by_id_loader
		// 	.load(active_period.subscription_id)
		// 	.await
		// 	.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError,
		// "failed to load subscription"))? 	.ok_or_else(||
		// ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load
		// subscription"))?;

		// let periods: Vec<_> = periods
		// 	.into_iter()
		// 	.filter(|p| p.subscription_id == active_period.subscription_id)
		// 	.collect();

		let product: SubscriptionProduct = global
			.subscription_products_loader
			.load(())
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription product"))?
			.map(|p| p.into_iter().next())
			.flatten()
			.ok_or_else(|| {
				ApiError::internal_server_error(ApiErrorCode::LoadError, "could not find subscription product")
			})?;

		let periods: Vec<_> = global
			.subscription_periods_by_user_id_loader
			.load(user_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription periods"))?
			.unwrap_or_default()
			.into_iter()
			.filter(|p| p.subscription_id.product_id == product.id)
			.collect();

		let age = sub_refresh_job::SubAge::new(&periods);

		// find relevant benefits
		let benefits: Vec<_> = product
			.benefits
			.into_iter()
			.filter_map(|b| match b.condition {
				SubscriptionBenefitCondition::Duration(d) => Some((b, d)),
				_ => None,
			})
			.collect();

		let traverse = &EntitlementEdgeGraphTraverse {
			inbound_loader: &global.entitlement_edge_inbound_loader,
			outbound_loader: &global.entitlement_edge_outbound_loader,
		};

		// follow the graph
		let edges = traverse
			.traversal(
				Direction::Outbound,
				benefits.iter().map(|(b, _)| EntitlementEdgeKind::SubscriptionBenefit {
					subscription_benefit_id: b.id,
				}),
			)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to traverse entitlement edges"))?;

		let mut badge_benefits: Vec<_> = edges
			.into_iter()
			.filter_map(|e| match (e.id.from, e.id.to) {
				(
					EntitlementEdgeKind::SubscriptionBenefit { subscription_benefit_id },
					EntitlementEdgeKind::Badge { badge_id },
				) => {
					let (b, d) = benefits.iter().find(|(b, _)| b.id == subscription_benefit_id)?;
					Some((b, *d, badge_id))
				}
				_ => None,
			})
			.collect();

		badge_benefits.sort_by_key(|(_, d, _)| *d);

		let mut badge_benefits = badge_benefits.into_iter().map(|(b, _, badge)| (b, badge));

		let mut current_badge = None;
		let mut next_badge = None;

		while let Some((b, badge)) = badge_benefits.next() {
			if age.meets_condition(&b.condition) {
				current_badge = Some(badge);
			} else {
				let next_period = if age.expected_end > chrono::Utc::now() { 1 } else { 0 };

				let (percentage, days_left) = match &b.condition {
					SubscriptionBenefitCondition::Duration(shared::database::duration::DurationUnit::Days(d)) => (
						(age.days + next_period) as f64 / *d as f64,
						(*d - (age.days + next_period)) as f64,
					),
					SubscriptionBenefitCondition::Duration(shared::database::duration::DurationUnit::Months(m)) => (
						(age.months + next_period) as f64 / *m as f64,
						(*m - (age.months + next_period)) as f64 * (365.25 / 12.0),
					),
					_ => unreachable!(),
				};

				next_badge = Some(BadgeProgressNextBadge {
					badge_id: badge,
					percentage,
					days_left,
				});

				break;
			}
		}

		Ok(BadgeProgress {
			current_badge_id: current_badge,
			next_badge,
		})
	}

	async fn monthly_paints(&self, ctx: &Context<'_>) -> Result<Vec<Paint>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let products = global
			.subscription_products_loader
			.load(())
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription products"))?
			.unwrap_or_default();

		// find relevant benefits
		let months_benefit_ids: Vec<_> = products
			.into_iter()
			.flat_map(|p| p.benefits)
			.filter(|b| match &b.condition {
				SubscriptionBenefitCondition::TimePeriod(tp) => {
					tp.start <= chrono::Utc::now() && tp.end > chrono::Utc::now()
				}
				_ => false,
			})
			.map(|b| EntitlementEdgeKind::SubscriptionBenefit {
				subscription_benefit_id: b.id,
			})
			.collect();

		let traverse = &EntitlementEdgeGraphTraverse {
			inbound_loader: &global.entitlement_edge_inbound_loader,
			outbound_loader: &global.entitlement_edge_outbound_loader,
		};

		// follow the graph
		let edges = traverse
			.traversal(Direction::Outbound, months_benefit_ids)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to traverse entitlement edges"))?;

		// find the paints
		let current_paints_ids: Vec<PaintId> = edges
			.into_iter()
			.filter_map(|e| match e.id.to {
				EntitlementEdgeKind::Paint { paint_id } => Some(paint_id),
				_ => None,
			})
			.collect();

		let current_paints = global
			.paint_by_id_loader
			.load_many(current_paints_ids)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load paints"))?
			.into_values()
			.map(|p| Paint::from_db(p, &global.config.api.cdn_origin))
			.collect();

		Ok(current_paints)
	}
}
