use std::collections::HashSet;
use std::sync::Arc;

use futures::TryStreamExt;
use shared::database::duration::DurationUnit;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy};
use shared::database::product::subscription::{
	Subscription, SubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy, SubscriptionState,
};
use shared::database::product::SubscriptionBenefitCondition;
use shared::database::queries::{filter, update};
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::ApiError;

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
pub async fn refresh(global: &Arc<Global>, subscription_id: &SubscriptionId) -> Result<(), ApiError> {
	let product = global
		.subscription_product_by_id_loader
		.load(subscription_id.product_id.clone())
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

	// load existing edges
	let outgoing: HashSet<_> = global
		.entitlement_edge_outbound_loader
		.load(EntitlementEdgeKind::Subscription {
			subscription_id: subscription_id.clone(),
		})
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default()
		.into_iter()
		.map(|e| e.id.to)
		.collect();

	let incoming: HashSet<_> = global
		.entitlement_edge_inbound_loader
		.load(EntitlementEdgeKind::Subscription {
			subscription_id: subscription_id.clone(),
		})
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default()
		.into_iter()
		.map(|e| e.id.from)
		.collect();

	// load all periods
	let periods: Vec<_> = SubscriptionPeriod::collection(&global.db)
		.find(filter::filter! {
			SubscriptionPeriod {
				#[query(serde)]
				subscription_id: subscription_id,
			}
		})
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to load subscription periods");
			ApiError::INTERNAL_SERVER_ERROR
		})?
		.try_collect()
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to collect subscription periods");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

	let mut new_edges = vec![];
	let mut remove_edges = vec![];

	for benefit in product.benefits {
		let (age_days, age_months) = sub_age(&periods);

		let is_fulfilled = match &benefit.condition {
			SubscriptionBenefitCondition::Duration(DurationUnit::Days(d)) => age_days >= (*d as isize),
			SubscriptionBenefitCondition::Duration(DurationUnit::Months(m)) => age_months >= (*m as isize),
			SubscriptionBenefitCondition::TimePeriod(tp) => periods.iter().any(|p| p.start <= tp.start && p.end >= tp.end),
		};

		let benefit_edge = EntitlementEdgeId {
			from: EntitlementEdgeKind::Subscription {
				subscription_id: subscription_id.clone(),
			},
			to: EntitlementEdgeKind::SubscriptionBenefit {
				subscription_benefit_id: benefit.id,
			},
			managed_by: Some(EntitlementEdgeManagedBy::Subscription {
				subscription_id: subscription_id.clone(),
			}),
		};

		if is_fulfilled && !outgoing.contains(&benefit_edge.to) {
			new_edges.push(benefit_edge);
		} else if !is_fulfilled && outgoing.contains(&benefit_edge.to) {
			remove_edges.push(benefit_edge);
		}
	}

	let now = chrono::Utc::now();
	let active_period = periods.iter().find(|p| p.start < now && p.end > now);

	let user_edge = EntitlementEdgeId {
		from: EntitlementEdgeKind::User {
			user_id: subscription_id.user_id,
		},
		to: EntitlementEdgeKind::Subscription {
			subscription_id: subscription_id.clone(),
		},
		managed_by: Some(EntitlementEdgeManagedBy::Subscription {
			subscription_id: subscription_id.clone(),
		}),
	};

	if let Some(active) = active_period {
		if !incoming.contains(&user_edge.from) {
			new_edges.push(user_edge);
		}

		let state = if matches!(
			active.created_by,
			SubscriptionPeriodCreatedBy::Invoice {
				cancel_at_period_end: true,
				..
			}
		) {
			SubscriptionState::CancelAtEnd
		} else {
			SubscriptionState::Active
		};

		Subscription::collection(&global.db)
			.update_one(
				filter::filter! {
					Subscription {
						#[query(rename = "_id", serde)]
						id: subscription_id,
					}
				},
				update::update! {
					#[query(set)]
					Subscription {
						#[query(rename = "_id", serde)]
						id: subscription_id,
						#[query(serde)]
						state,
						updated_at: chrono::Utc::now(),
					}
				},
			)
			.upsert(true)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update subscription");
				ApiError::INTERNAL_SERVER_ERROR
			})?;
	} else {
		if incoming.contains(&user_edge.from) {
			remove_edges.push(user_edge);
		}

		Subscription::collection(&global.db)
			.update_one(
				filter::filter! {
					Subscription {
						#[query(rename = "_id", serde)]
						id: subscription_id,
					}
				},
				update::update! {
					#[query(set)]
					Subscription {
						#[query(rename = "_id", serde)]
						id: subscription_id,
						#[query(serde)]
						state: SubscriptionState::Ended,
						updated_at: chrono::Utc::now(),
					}
				},
			)
			.upsert(true)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update subscription");
				ApiError::INTERNAL_SERVER_ERROR
			})?;
	}

	if !remove_edges.is_empty() {
		EntitlementEdge::collection(&global.db)
			.delete_many(filter::filter! {
				EntitlementEdge {
					#[query(rename = "_id", selector = "in", serde)]
					id: remove_edges,
				}
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to delete entitlement edges");
				ApiError::INTERNAL_SERVER_ERROR
			})?;
	}

	if !new_edges.is_empty() {
		EntitlementEdge::collection(&global.db)
			.insert_many(new_edges.into_iter().map(|id| EntitlementEdge { id }))
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert entitlement edges");
				ApiError::INTERNAL_SERVER_ERROR
			})?;
	}

	Ok(())
}
