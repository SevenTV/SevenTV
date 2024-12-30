use std::collections::HashSet;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;

use chrono::TimeZone;
use futures::TryStreamExt;
use mongodb::options::FindOneOptions;
use shared::database::duration::DurationUnit;
use shared::database::emote_set::{EmoteSet, EmoteSetId, EmoteSetKind};
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy};
use shared::database::product::special_event::SpecialEventId;
use shared::database::product::subscription::{Subscription, SubscriptionId, SubscriptionPeriod, SubscriptionState};
use shared::database::product::{ProductId, SubscriptionBenefitCondition};
use shared::database::queries::{filter, update};
use shared::database::user::{User, UserId, UserStyle};
use shared::database::MongoCollection;
use shared::event::{InternalEvent, InternalEventData, InternalEventEmoteSetData};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::transactions::{transaction_with_mutex, TransactionError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubAge {
	pub extra: chrono::Duration,
	pub months: i32,
	pub days: i32,
	pub periods: Vec<StartEnd>,
	pub expected_end: chrono::DateTime<chrono::Utc>,
}

impl SubAge {
	pub fn new(periods: &[SubscriptionPeriod]) -> Self {
		// We need to sum up all the time so that we can calculate the age of the
		// subscription. We want to make sure there are no overlapping periods so we
		// dont have duplicate time.
		let now = chrono::Utc::now();

		let expected_end = periods.iter().map(|p| p.end).max().unwrap_or(now);

		let mut combined_periods = periods
			.iter()
			.filter(|p| p.start < now)
			.map(|p| StartEnd {
				start: p.start,
				end: p.end.min(now),
			})
			.collect::<Vec<_>>();

		combined_periods.sort_by(|a, b| a.start.cmp(&b.start));

		let merged_periods: Vec<StartEnd> = combined_periods.into_iter().fold(Vec::new(), |mut acc, period| {
			if acc.is_empty() {
				acc.push(period);
				return acc;
			}

			let last = acc.last_mut().unwrap();
			if last.end >= period.start {
				last.end = period.end.max(last.end);
			} else {
				acc.push(period);
			}

			acc
		});

		let days = merged_periods
			.iter()
			.map(|p| (p.end.min(now) - p.start))
			.sum::<chrono::Duration>()
			.num_days() as i32;

		let months = days as f64 / (365.25 / 12.0);
		let extra = chrono::Duration::days((months.fract() * 30.44).round() as i64);
		let months = months as i32;

		SubAge {
			extra,
			months,
			days,
			periods: merged_periods,
			expected_end,
		}
	}

	pub fn meets_condition(&self, condition: &SubscriptionBenefitCondition) -> bool {
		// Consider the Subscription, if their sub is set to end in the future then they
		// should get the entitlements for the period that they are currently in.
		// if you sub to twitch you are given the 1 month sub badge even though you
		// havent subbed for the entire month yet, this is because the sub is set to end
		// in the future. However if you unsub at the end of your term you would have
		// completed the month and wouldnt get the next badge because your sub has
		// ended. Then once you start subbing again you would get the next badge.
		let next_period = if self.expected_end > chrono::Utc::now() { 1 } else { 0 };

		match condition {
			SubscriptionBenefitCondition::Duration(DurationUnit::Days(d)) => self.days + next_period >= *d,
			SubscriptionBenefitCondition::Duration(DurationUnit::Months(m)) => self.months + next_period >= *m,
			SubscriptionBenefitCondition::TimePeriod(tp) => self.periods.iter().any(|p| {
				(p.start <= tp.start && p.end >= tp.start)
					|| (p.start <= tp.end && p.end >= tp.end)
					|| (p.start >= tp.start && p.end <= tp.end)
			}),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartEnd {
	pub start: chrono::DateTime<chrono::Utc>,
	pub end: chrono::DateTime<chrono::Utc>,
}

/// Grants entitlements for a subscription.
pub async fn refresh(global: &Arc<Global>, subscription_id: SubscriptionId) -> Result<(), ApiError> {
	global
		.mutex
		.acquire(format!("mutex:subscription:refresh:{subscription_id}"), || async {
			let product = global
				.subscription_product_by_id_loader
				.load(subscription_id.product_id)
				.await
				.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load product"))?
				.ok_or_else(|| ApiError::internal_server_error(ApiErrorCode::LoadError, "product not found"))?;

			// load existing edges
			let outgoing: HashSet<_> = global
				.entitlement_edge_outbound_loader
				.load(EntitlementEdgeKind::Subscription { subscription_id })
				.await
				.map_err(|_| {
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription entitlements")
				})?
				.unwrap_or_default()
				.into_iter()
				.filter(|e| e.id.managed_by == Some(EntitlementEdgeManagedBy::Subscription { subscription_id }))
				.map(|e| e.id.to)
				.collect();

			let incoming: HashSet<_> = global
				.entitlement_edge_inbound_loader
				.load(EntitlementEdgeKind::Subscription { subscription_id })
				.await
				.map_err(|_| {
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription entitlements")
				})?
				.unwrap_or_default()
				.into_iter()
				.filter(|e| e.id.managed_by == Some(EntitlementEdgeManagedBy::Subscription { subscription_id }))
				.map(|e| e.id.from)
				.collect();

			// load all periods
			let periods: Vec<_> = SubscriptionPeriod::collection(&global.db)
				.find(filter::filter! {
					SubscriptionPeriod {
						#[query(serde)]
						subscription_id,
					}
				})
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to load subscription periods");
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription periods")
				})?
				.try_collect()
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to collect subscription periods");
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to collect subscription periods")
				})?;

			let mut new_edges = vec![];
			let mut remove_edges = vec![];
			let sub_age = SubAge::new(&periods);

			for benefit in product.benefits {
				let is_fulfilled = sub_age.meets_condition(&benefit.condition);

				let benefit_edge = EntitlementEdgeId {
					from: EntitlementEdgeKind::Subscription { subscription_id },
					to: EntitlementEdgeKind::SubscriptionBenefit {
						subscription_benefit_id: benefit.id,
					},
					managed_by: Some(EntitlementEdgeManagedBy::Subscription { subscription_id }),
				};

				if is_fulfilled && !outgoing.contains(&benefit_edge.to) {
					new_edges.push(benefit_edge);
				} else if !is_fulfilled && outgoing.contains(&benefit_edge.to) {
					remove_edges.push(benefit_edge);
				}
			}

			let xmas_gift = handle_xmas_2024_gift(global, subscription_id.user_id).await?;
			for add in xmas_gift.adds {
				new_edges.push(EntitlementEdgeId {
					from: EntitlementEdgeKind::Subscription { subscription_id },
					to: EntitlementEdgeKind::SpecialEvent { special_event_id: add },
					managed_by: Some(EntitlementEdgeManagedBy::Subscription { subscription_id }),
				});
			}

			for remove in xmas_gift.removes {
				remove_edges.push(EntitlementEdgeId {
					from: EntitlementEdgeKind::Subscription { subscription_id },
					to: EntitlementEdgeKind::SpecialEvent {
						special_event_id: remove,
					},
					managed_by: Some(EntitlementEdgeManagedBy::Subscription { subscription_id }),
				});
			}

			let now = chrono::Utc::now();
			let grace_period = now + chrono::Duration::days(2);
			let active_periods = periods
				.iter()
				.filter(|p| p.start < now && (p.end > now || (p.auto_renew && p.end > grace_period)))
				.collect::<Vec<_>>();

			let user_edge = EntitlementEdgeId {
				from: EntitlementEdgeKind::User {
					user_id: subscription_id.user_id,
				},
				to: EntitlementEdgeKind::Subscription { subscription_id },
				managed_by: Some(EntitlementEdgeManagedBy::Subscription { subscription_id }),
			};

			if !active_periods.is_empty() {
				if !incoming.contains(&user_edge.from) {
					new_edges.push(user_edge);
				}

				let state = if active_periods.iter().any(|period| period.auto_renew) {
					SubscriptionState::Active
				} else {
					SubscriptionState::CancelAtEnd
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
								#[query(serde)]
								state,
								ended_at: &None,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							},
							#[query(set_on_insert)]
							Subscription {
								#[query(rename = "_id", serde)]
								id: subscription_id,
								created_at: chrono::Utc::now(),
							}
						},
					)
					.upsert(true)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to update subscription");
						ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to update subscription")
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
								#[query(serde)]
								state: SubscriptionState::Ended,
								ended_at: Some(sub_age.expected_end),
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							},
							#[query(set_on_insert)]
							Subscription {
								#[query(rename = "_id", serde)]
								id: subscription_id,
								created_at: chrono::Utc::now(),
							}
						},
					)
					.upsert(true)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to update subscription");
						ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to update subscription")
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
						ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to delete entitlement edges")
					})?;
			}

			if !new_edges.is_empty() {
				EntitlementEdge::collection(&global.db)
					.insert_many(new_edges.into_iter().map(|id| EntitlementEdge { id }))
					.with_options(mongodb::options::InsertManyOptions::builder().ordered(false).build())
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to delete entitlement edges");
						ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to delete entitlement edges")
					})?;
			}

			let personal_emote_set_id = EmoteSet::collection(&global.db)
				.find_one(filter::filter! {
					EmoteSet {
						owner_id: subscription_id.user_id,
						#[query(serde)]
						kind: EmoteSetKind::Personal,
					}
				})
				.with_options(FindOneOptions::builder().sort(bson::doc! { "_id": -1 }).build())
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to update emote set");
					ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to update emote set")
				})?
				.map(|set| set.id);

			let personal_emote_set_id = if let Some(personal_emote_set_id) = personal_emote_set_id {
				personal_emote_set_id
			} else {
				transaction_with_mutex(
					global,
					Some(format!("mutex:user:sub:personal:{}", subscription_id.user_id).into()),
					|mut tx| async move {
						if let Some(set) = tx
							.find_one(
								filter::filter! {
									EmoteSet {
										owner_id: subscription_id.user_id,
										#[query(serde)]
										kind: EmoteSetKind::Personal,
									}
								},
								Some(FindOneOptions::builder().sort(bson::doc! { "_id": -1 }).build()),
							)
							.await?
						{
							return Ok(set.id);
						}

						let set = EmoteSet {
							id: EmoteSetId::new(),
							name: "Personal Emote Set".to_string(),
							owner_id: Some(subscription_id.user_id),
							kind: EmoteSetKind::Personal,
							updated_at: chrono::Utc::now(),
							origin_config: None,
							capacity: Some(5), /* TODO: this is hard coded however we should likely get this from the sub
							                    * product */
							description: None,
							emotes: vec![],
							emotes_changed_since_reindex: false,
							tags: vec![],
							search_updated_at: None,
						};

						tx.insert_one::<EmoteSet>(&set, None).await?;

						let id = set.id;

						tx.register_event(InternalEvent {
							actor: None,
							session_id: None,
							timestamp: chrono::Utc::now(),
							data: InternalEventData::EmoteSet {
								after: set,
								data: InternalEventEmoteSetData::Create,
							},
						})?;

						Ok::<_, TransactionError<Infallible>>(id)
					},
				)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to create personal emote set");
					ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to create personal emote set")
				})?
			};

			User::collection(&global.db)
				.update_one(
					filter::filter! {
						User {
							#[query(rename = "_id")]
							id: subscription_id.user_id,
							#[query(flatten)]
							style: UserStyle {
								personal_emote_set_id: &None,
							},
						}
					},
					update::update! {
						#[query(set)]
						User {
							#[query(flatten)]
							style: UserStyle {
								personal_emote_set_id,
							},
						},
					},
				)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to update user");
					ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to update user")
				})?;

			Ok(())
		})
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to acquire mutex");
			ApiError::internal_server_error(ApiErrorCode::Unknown, "failed to acquire mutex")
		})?
}

struct Xmas2024Gift {
	adds: Vec<SpecialEventId>,
	removes: Vec<SpecialEventId>,
}

struct Level {
	event_id: SpecialEventId,
	count: i32,
}

struct Event {
	start: chrono::DateTime<chrono::Utc>,
	end: chrono::DateTime<chrono::Utc>,
	levels: Vec<Level>,
}

async fn handle_xmas_2024_gift(global: &Arc<Global>, customer_id: UserId) -> Result<Xmas2024Gift, ApiError> {
	// Count the number of subs the user has gifted.
	const XMAS_1_SUB_ID: &str = "0193b680-0de9-25b4-dd0d-21b8ab811bc0";
	const XMAS_10_SUBS_ID: &str = "0193F0E3-828A-13C2-1E24-17CF874748DF";
	const MINECRAFT_5_SUBS_ID: &str = "0193ef9e-ffa6-8c60-ceb6-279134b5b32a";
	const MINECRAFT_10_SUBS_ID: &str = "0193ef9f-2ed9-2e29-48e2-ad78887dfab9";

	let events = [
		Event {
			start: chrono::Utc.with_ymd_and_hms(2024, 12, 14, 0, 0, 0).unwrap(),
			end: chrono::Utc.with_ymd_and_hms(2024, 12, 31, 0, 0, 0).unwrap(),
			levels: vec![Level {
				event_id: SpecialEventId::from_str(XMAS_1_SUB_ID).unwrap(),
				count: 1,
			}],
		},
		Event {
			start: chrono::Utc.with_ymd_and_hms(2024, 12, 14, 0, 0, 0).unwrap(),
			end: chrono::Utc.with_ymd_and_hms(2024, 12, 28, 0, 0, 0).unwrap(),
			levels: vec![Level {
				event_id: SpecialEventId::from_str(XMAS_10_SUBS_ID).unwrap(),
				count: 10,
			}],
		},
		Event {
			start: chrono::Utc.with_ymd_and_hms(2024, 12, 28, 0, 0, 0).unwrap(),
			end: chrono::Utc.with_ymd_and_hms(2024, 12, 31, 0, 0, 0).unwrap(),
			levels: vec![
				Level {
					event_id: SpecialEventId::from_str(MINECRAFT_5_SUBS_ID).unwrap(),
					count: 5,
				},
				Level {
					event_id: SpecialEventId::from_str(MINECRAFT_10_SUBS_ID).unwrap(),
					count: 10,
				},
			],
		},
	];

	let gifted_subs = SubscriptionPeriod::collection(&global.db)
		.find(filter::filter! {
			SubscriptionPeriod {
				gifted_by: customer_id,
			}
		})
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to find gifted subs");
			ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to find gifted subs")
		})?
		.try_collect::<Vec<_>>()
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to collect gifted subs");
			ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to collect gifted subs")
		})?;

	// Filter out any subs that are not in the xmas event
	const MONTH_SUB_PRODUCT_ID: &str = "price_1JWQ2QCHxsWbK3R31cZkaocV"; // = 1 gift
	const YEAR_SUB_PRODUCT_ID: &str = "price_1JWQ2RCHxsWbK3R3a6emz76a"; // = 10 gifts

	let month_product_id = ProductId::from(stripe::PriceId::from_str(MONTH_SUB_PRODUCT_ID).unwrap());
	let year_product_id = ProductId::from(stripe::PriceId::from_str(YEAR_SUB_PRODUCT_ID).unwrap());

	let mut xmas_gift = Xmas2024Gift {
		adds: vec![],
		removes: vec![],
	};

	for event in events {
		let mut gift_count = 0;
		for gifted_sub in &gifted_subs {
			if gifted_sub.id.timestamp() >= event.start && gifted_sub.id.timestamp() <= event.end {
				if gifted_sub.product_id == month_product_id {
					gift_count += 1;
				} else if gifted_sub.product_id == year_product_id {
					gift_count += 10;
				}
			}
		}

		for level in &event.levels {
			if gift_count >= level.count {
				xmas_gift.adds.push(level.event_id);
			} else {
				xmas_gift.removes.push(level.event_id);
			}
		}
	}

	Ok(xmas_gift)
}
