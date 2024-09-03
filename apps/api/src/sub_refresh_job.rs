use std::sync::Arc;

use shared::database::duration::DurationUnit;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy};
use shared::database::product::subscription::{SubscriptionId, SubscriptionPeriod};
use shared::database::product::{SubscriptionBenefit, SubscriptionBenefitCondition};
use shared::database::queries::filter;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub fn sub_age_days(periods: &[SubscriptionPeriod]) -> isize {
	sub_age(periods).0
}

pub fn sub_age(periods: &[SubscriptionPeriod]) -> (isize, isize) {
	periods
		.iter()
		.map(|p| {
			let diff = date_component::date_component::calculate(&p.start, &p.end);
			(diff.interval_days, diff.month)
		})
		.fold((0, 0), |(days, month), diff| (days + diff.0, month + diff.1))
}

/// Grants entitlements for a subscription.
pub async fn refresh_entitlements(
	tx: &mut TransactionSession<'_, ApiError>,
	subscription_id: &SubscriptionId,
	benefits: &[SubscriptionBenefit],
) -> TransactionResult<(), ApiError> {
	// first revoke all entitlements
	revoke_entitlements(tx, subscription_id).await?;

	// load all periods
	let periods = tx
		.find(
			filter::filter! {
				SubscriptionPeriod {
					#[query(serde)]
					subscription_id: subscription_id,
				}
			},
			None,
		)
		.await?;

	let now = chrono::Utc::now();
	let active = periods.iter().any(|p| p.start < now && p.end > now);

	if active {
		let mut edges = vec![];

		// always insert the edge from the user to the subscription
		edges.push(EntitlementEdgeId {
			from: EntitlementEdgeKind::User {
				user_id: subscription_id.user_id,
			},
			to: EntitlementEdgeKind::Subscription {
				subscription_id: subscription_id.clone(),
			},
			managed_by: Some(EntitlementEdgeManagedBy::Subscription {
				subscription_id: subscription_id.clone(),
			}),
		});

		let (age_days, age_months) = sub_age(&periods);

		for benefit in benefits {
			let is_fulfilled = match &benefit.condition {
				SubscriptionBenefitCondition::Duration(DurationUnit::Days(d)) => age_days >= (*d as isize),
				SubscriptionBenefitCondition::Duration(DurationUnit::Months(m)) => age_months >= (*m as isize),
				SubscriptionBenefitCondition::TimePeriod(tp) => {
					periods.iter().any(|p| p.start <= tp.start && p.end >= tp.end)
				}
			};

			if is_fulfilled {
				edges.push(EntitlementEdgeId {
					from: EntitlementEdgeKind::Subscription {
						subscription_id: subscription_id.clone(),
					},
					to: EntitlementEdgeKind::SubscriptionBenefit {
						subscription_benefit_id: benefit.id,
					},
					managed_by: Some(EntitlementEdgeManagedBy::Subscription {
						subscription_id: subscription_id.clone(),
					}),
				});
			}
		}

		tx.insert_many(edges.into_iter().map(|id| EntitlementEdge { id }), None)
			.await?;
	}

	Ok(())
}

/// Grants entitlements for a subscription.
pub async fn revoke_entitlements(
	tx: &mut TransactionSession<'_, ApiError>,
	subscription_id: &SubscriptionId,
) -> TransactionResult<(), ApiError> {
	tx.delete(
		filter::filter! {
			EntitlementEdge {
				#[query(rename = "_id", flatten)]
				id: EntitlementEdgeId {
					#[query(serde)]
					managed_by: Some(EntitlementEdgeManagedBy::Subscription { subscription_id: subscription_id.clone() }),
				}
			}
		},
		None,
	)
	.await?;

	Ok(())
}

/// Call this function from the refresh cron job.
pub async fn refresh(
	global: &Arc<Global>,
	tx: &mut TransactionSession<'_, ApiError>,
	subscription_id: &SubscriptionId,
) -> TransactionResult<(), ApiError> {
	let product = global
		.subscription_product_by_id_loader
		.load(subscription_id.product_id.clone())
		.await
		.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
		.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

	refresh_entitlements(tx, subscription_id, &product.benefits).await?;

	Ok(())
}
