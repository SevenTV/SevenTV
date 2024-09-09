use std::str::FromStr;
use std::sync::Arc;

use fnv::{FnvHashMap, FnvHashSet};
use mongodb::options::InsertManyOptions;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy};
use shared::database::product::subscription::SubscriptionId;
use shared::database::product::SubscriptionProductId;
use shared::database::MongoCollection;

use super::prices::NEW_PRODUCT_ID;
use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types::EntitlementData;
use crate::{error, types};

pub struct EntitlementsJob {
	global: Arc<Global>,
	skip_entitlements: FnvHashMap<EntitlementEdgeKind, (EntitlementEdgeKind, Option<EntitlementEdgeManagedBy>, bool)>,
	edges: FnvHashSet<EntitlementEdge>,
}

fn skip_entitlements() -> impl Iterator<Item = (EntitlementEdgeKind, (EntitlementEdgeKind, Option<EntitlementEdgeManagedBy>, bool))>
{
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
				.map(move |to| (to, (EntitlementEdgeKind::Role { role_id: r.id }, None, r.id != "62f99d0ce46eb00e438a6984".parse().unwrap())))
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

impl Job for EntitlementsJob {
	type T = types::Entitlement;

	const NAME: &'static str = "transfer_entitlements";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			EntitlementEdge::collection(global.target_db()).drop().await?;
			let indexes = EntitlementEdge::indexes();
			if !indexes.is_empty() {
				EntitlementEdge::collection(global.target_db())
					.create_indexes(indexes)
					.await?;
			}
		}

		Ok(Self {
			global,
			skip_entitlements: FnvHashMap::from_iter(skip_entitlements()),
			edges: FnvHashSet::from_iter(custom_edges()),
		})
	}

	async fn collection(&self) -> Option<mongodb::Collection<Self::T>> {
		Some(self.global.source_db().collection("entitlements"))
	}

	async fn process(&mut self, entitlement: Self::T) -> ProcessOutcome {
		let Some(user_id) = entitlement.user_id else {
			return ProcessOutcome::default();
		};

		let to = match entitlement.data {
			EntitlementData::Badge { ref_id, .. } => EntitlementEdgeKind::Badge { badge_id: ref_id.into() },
			EntitlementData::Paint { ref_id, .. } => EntitlementEdgeKind::Paint { paint_id: ref_id.into() },
			// ignore role & emote set entitlements because they are handled by the user job
			_ => return ProcessOutcome::default(),
		};

		if let Some((custom_edge, managed_by, ignore)) = self.skip_entitlements.get(&to) {
			if !ignore {
				self.edges.insert(EntitlementEdge {
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
			self.edges.insert(EntitlementEdge {
				id: EntitlementEdgeId {
					from: EntitlementEdgeKind::User { user_id: user_id.into() },
					to,
					managed_by: None,
				},
			});
		}

		ProcessOutcome::default()
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing entitlements job");

		let mut outcome = ProcessOutcome::default();

		match EntitlementEdge::collection(self.global.target_db())
			.insert_many(&self.edges)
			.with_options(InsertManyOptions::builder().ordered(false).build())
			.await
		{
			Ok(res) => {
				outcome.inserted_rows += res.inserted_ids.len() as u64;
				if res.inserted_ids.len() != self.edges.len() {
					outcome.errors.push(error::Error::InsertMany);
				}
			}
			Err(e) => outcome.errors.push(e.into()),
		}

		self.global.entitlement_job_token().cancel();

		outcome
	}
}
