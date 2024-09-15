use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use shared::database::badge::BadgeId;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy};
use shared::database::paint::PaintId;
use shared::database::product::subscription::SubscriptionId;
use shared::database::product::SubscriptionProductId;
use shared::database::MongoCollection;

use super::prices::NEW_PRODUCT_ID;
use super::{JobOutcome, ProcessOutcome};
use crate::global::Global;
use crate::types;
use crate::types::EntitlementData;

fn skip_entitlements() -> impl Iterator<
	Item = (
		EntitlementEdgeKind,
		(EntitlementEdgeKind, Option<EntitlementEdgeManagedBy>, bool),
	),
> {
	super::subscriptions::benefits::sub_badges_benefits()
		.into_iter()
		.flat_map(|b| {
			b.entitlements.into_iter().map(move |to| {
				(
					to,
					(
						EntitlementEdgeKind::SubscriptionBenefit {
							subscription_benefit_id: b.benefit.id,
						},
						None,
						false,
					),
				)
			})
		})
		.chain(
			super::subscriptions::benefits::sub_monthly_benefits()
				.into_iter()
				.flat_map(|b| {
					b.entitlements.into_iter().map(move |to| {
						(
							to,
							(
								EntitlementEdgeKind::SubscriptionBenefit {
									subscription_benefit_id: b.benefit.id,
								},
								None,
								false,
							),
						)
					})
				}),
		)
		.chain(super::subscriptions::benefits::role_entitlements().into_iter().flat_map(|r| {
			r.entitlements
				.into_iter()
				// Ignore all other role inserts because they are handled by the user job.
				// This is a new role called the `Translator` role which is not handled by the user job.
				.map(move |to| {
					(
						to,
						(
							EntitlementEdgeKind::Role { role_id: r.id },
							None,
							r.id != "62f99d0ce46eb00e438a6984".parse().unwrap(),
						),
					)
				})
		}))
		.chain(super::subscriptions::benefits::special_events().into_iter().flat_map(|s| {
			s.entitlements.into_iter().map(move |to| {
				(
					to,
					(
						EntitlementEdgeKind::SpecialEvent {
							special_event_id: s.special_event.id,
						},
						Some(s.managed_by.clone()),
						false,
					),
				)
			})
		}))
}

fn custom_edges() -> impl IntoIterator<Item = EntitlementEdge> {
	super::subscriptions::benefits::sub_badges_benefits()
		.into_iter()
		.flat_map(|b| {
			b.entitlements.into_iter().map(move |to| EntitlementEdge {
				id: EntitlementEdgeId {
					to,
					managed_by: None,
					from: EntitlementEdgeKind::SubscriptionBenefit {
						subscription_benefit_id: b.benefit.id,
					},
				},
			})
		})
		.chain(
			super::subscriptions::benefits::sub_monthly_benefits()
				.into_iter()
				.flat_map(|b| {
					b.entitlements.into_iter().map(move |to| EntitlementEdge {
						id: EntitlementEdgeId {
							to,
							managed_by: None,
							from: EntitlementEdgeKind::SubscriptionBenefit {
								subscription_benefit_id: b.benefit.id,
							},
						},
					})
				}),
		)
		.chain(super::subscriptions::benefits::role_entitlements().into_iter().flat_map(|r| {
			r.entitlements.into_iter().map(move |to| EntitlementEdge {
				id: EntitlementEdgeId {
					to,
					managed_by: None,
					from: EntitlementEdgeKind::Role { role_id: r.id },
				},
			})
		}))
		.chain(super::subscriptions::benefits::special_events().into_iter().flat_map(|s| {
			s.entitlements.into_iter().map(move |to| EntitlementEdge {
				id: EntitlementEdgeId {
					to,
					managed_by: None,
					from: EntitlementEdgeKind::SpecialEvent {
						special_event_id: s.special_event.id,
					},
				},
			})
		}))
		.chain(std::iter::once(EntitlementEdge {
			id: EntitlementEdgeId {
				from: EntitlementEdgeKind::GlobalDefaultEntitlementGroup,
				to: EntitlementEdgeKind::Role {
					role_id: "62b48deb791a15a25c2a0354".parse().unwrap(),
				},
				managed_by: None,
			},
		}))
}

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub edges: &'a mut HashSet<EntitlementEdge>,
	pub badge_filter: Box<dyn Fn(BadgeId) -> bool + 'a>,
	pub paint_filter: Box<dyn Fn(PaintId) -> bool + 'a>,
}

pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("entitlements");

	let RunInput {
		global,
		edges,
		badge_filter,
		paint_filter,
	} = input;

	let mut cursor = global
		.source_db()
		.collection::<types::Entitlement>("entitlements")
		.find(bson::doc! {})
		.await
		.context("query")?;

	let skipped = HashMap::from_iter(skip_entitlements());

	edges.extend(custom_edges());

	while let Some(entitlement) = cursor.next().await {
		match entitlement {
			Ok(entitlement) => {
				outcome += process(ProcessInput {
					edges,
					entitlement,
					skipped: &skipped,
					badge_filter: &badge_filter,
					paint_filter: &paint_filter,
				});
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}

struct ProcessInput<'a> {
	entitlement: types::Entitlement,
	badge_filter: &'a Box<dyn Fn(BadgeId) -> bool + 'a>,
	paint_filter: &'a Box<dyn Fn(PaintId) -> bool + 'a>,
	skipped: &'a HashMap<EntitlementEdgeKind, (EntitlementEdgeKind, Option<EntitlementEdgeManagedBy>, bool)>,
	edges: &'a mut HashSet<EntitlementEdge>,
}

fn process(input: ProcessInput<'_>) -> ProcessOutcome {
	let ProcessInput {
		entitlement,
		edges,
		skipped,
		badge_filter,
		paint_filter,
	} = input;

	let Some(user_id) = entitlement.user_id else {
		return ProcessOutcome::default();
	};

	let to = match entitlement.data {
		EntitlementData::Badge { ref_id, .. } => {
			let badge_id = ref_id.into();

			if !badge_filter(&badge_id) {
				return ProcessOutcome::default();
			}

			EntitlementEdgeKind::Badge { badge_id: ref_id.into() }
		}
		EntitlementData::Paint { ref_id, .. } => {
			let paint_id = ref_id.into();

			if !paint_filter(&paint_id) {
				return ProcessOutcome::default();
			}

			EntitlementEdgeKind::Paint { paint_id: ref_id.into() }
		}
		// ignore role & emote set entitlements because they are handled by the user job
		_ => return ProcessOutcome::default(),
	};

	if let Some((custom_edge, managed_by, ignore)) = skipped.get(&to) {
		if !ignore {
			edges.insert(EntitlementEdge {
				id: EntitlementEdgeId {
					from: EntitlementEdgeKind::Subscription {
						subscription_id: SubscriptionId {
							user_id: user_id.into(),
							product_id: SubscriptionProductId::from_str(NEW_PRODUCT_ID).unwrap(),
						},
					},
					to: custom_edge.clone(),
					managed_by: managed_by.clone(),
				},
			});
		}
	} else {
		edges.insert(EntitlementEdge {
			id: EntitlementEdgeId {
				from: EntitlementEdgeKind::User { user_id: user_id.into() },
				to,
				managed_by: None,
			},
		});
	}

	ProcessOutcome::default()
}
