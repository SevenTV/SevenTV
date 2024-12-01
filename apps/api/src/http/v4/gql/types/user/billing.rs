use std::sync::Arc;

use async_graphql::Context;
use shared::database::badge::BadgeId;
use shared::database::entitlement::EntitlementEdgeKind;
use shared::database::entitlement_edge::EntitlementEdgeGraphTraverse;
use shared::database::graph::{Direction, GraphTraverse};
use shared::database::product::{SubscriptionBenefitCondition, SubscriptionProductId};
use shared::database::user::UserId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v4::gql::types::{Badge, SubscriptionPeriod};
use crate::sub_refresh_job;

#[derive(Debug, Clone)]
pub struct Billing {
	pub user_id: UserId,
	pub product_id: SubscriptionProductId,
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct BadgeProgress {
	pub current_badge_id: Option<BadgeId>,
	pub next_badge: Option<BadgeProgressNextBadge>,
}

#[async_graphql::ComplexObject]
impl BadgeProgress {
	#[tracing::instrument(skip_all, name = "BadgeProgress::current_badge")]
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
	#[tracing::instrument(skip_all, name = "BadgeProgressNextBadge::badge")]
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

#[derive(async_graphql::SimpleObject)]
pub struct SubscriptionInfo {
	pub total_days: i32,
	pub active_period: Option<SubscriptionPeriod>,
}

#[async_graphql::Object]
impl Billing {
	#[tracing::instrument(skip_all, name = "Billing::badge_progress")]
	async fn badge_progress(&self, ctx: &Context<'_>) -> Result<BadgeProgress, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let product = global
			.subscription_product_by_id_loader
			.load(self.product_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription product"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "could not find subscription product"))?;

		let periods: Vec<_> = global
			.subscription_periods_by_user_id_loader
			.load(self.user_id)
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
				let current_benefit_days = match b.condition {
					SubscriptionBenefitCondition::Duration(shared::database::duration::DurationUnit::Days(d)) => d as f64,
					SubscriptionBenefitCondition::Duration(shared::database::duration::DurationUnit::Months(m)) => {
						m as f64 * (365.25 / 12.0)
					}
					_ => unreachable!(),
				};

				current_badge = Some((badge, current_benefit_days));
			} else {
				let (next_benefit_days, total_days) = match &b.condition {
					SubscriptionBenefitCondition::Duration(shared::database::duration::DurationUnit::Days(d)) => {
						(*d as f64, age.days as f64 + 1.0)
					}
					SubscriptionBenefitCondition::Duration(shared::database::duration::DurationUnit::Months(m)) => {
						let nb_days = *m as f64 * (365.25 / 12.0);
						let total_days = (age.months as f64 + 1.0) * (365.25 / 12.0) + age.extra.num_days() as f64;
						(nb_days, total_days)
					}
					_ => unreachable!(),
				};

				let current_benefit_days = current_badge.map(|(_, d)| d).unwrap_or(365.25 / 12.0);

				next_badge = Some(BadgeProgressNextBadge {
					badge_id: badge,
					percentage: (total_days - current_benefit_days) / (next_benefit_days - current_benefit_days),
					days_left: next_benefit_days - total_days,
				});

				break;
			}
		}

		Ok(BadgeProgress {
			current_badge_id: current_badge.map(|(badge, _)| badge),
			next_badge,
		})
	}

	#[tracing::instrument(skip_all, name = "Billing::subscription_info")]
	async fn subscription_info(&self, ctx: &Context<'_>) -> Result<SubscriptionInfo, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let periods: Vec<_> = global
			.subscription_periods_by_user_id_loader
			.load(self.user_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription periods"))?
			.unwrap_or_default()
			.into_iter()
			.filter(|p| p.subscription_id.product_id == self.product_id)
			.collect();

		let age = sub_refresh_job::SubAge::new(&periods);

		let active_period = periods
			.iter()
			.find(|p| p.start < chrono::Utc::now() && p.end > chrono::Utc::now())
			.cloned();

		Ok(SubscriptionInfo {
			total_days: age.days,
			active_period: active_period.map(Into::into),
		})
	}
}
