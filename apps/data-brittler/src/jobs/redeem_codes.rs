use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use shared::database::entitlement::EntitlementEdgeKind;
use shared::database::paint::PaintId;
use shared::database::product::codes::{CodeEffect, RedeemCode, RedeemCodeId, RedeemCodeSubscriptionEffect};
use shared::database::product::special_event::SpecialEventId;
use shared::database::product::SubscriptionProductId;
use shared::database::user::UserId;

use super::prices::NEW_SUBSCRIPTION_PRODUCT_ID;
use super::subscriptions::benefits::special_events;
use super::{JobOutcome, ProcessOutcome};
use crate::error;
use crate::global::Global;
use crate::types::{self, EntitlementData};

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub redeem_codes: &'a mut HashMap<String, RedeemCode>,
}

pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("redeem_codes");

	let RunInput { global, redeem_codes } = input;

	let mut cursor = global
		.egvault_source_db
		.collection::<types::RedeemCode>("promotions")
		.find(bson::doc! {})
		.await
		.context("query")?;

	let mut paint_events = HashMap::new();

	special_events().into_iter().for_each(|s| {
		s.entitlements.iter().for_each(|e| {
			if let EntitlementEdgeKind::Paint { paint_id } = e {
				paint_events.insert(*paint_id, s.special_event.id);
			}
		});
	});

	while let Some(redeem_code) = cursor.next().await {
		match redeem_code {
			Ok(redeem_code) => {
				outcome += process(ProcessInput {
					redeem_codes,
					redeem_code,
					paint_events: &paint_events,
				});
				outcome.processed_documents += 1;
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}

struct ProcessInput<'a> {
	redeem_codes: &'a mut HashMap<String, RedeemCode>,
	paint_events: &'a HashMap<PaintId, SpecialEventId>,
	redeem_code: types::RedeemCode,
}

fn process(input: ProcessInput) -> ProcessOutcome {
	let outcome = ProcessOutcome::default();

	let ProcessInput {
		redeem_codes,
		redeem_code,
		paint_events,
	} = input;

	if let Some(redeem_code) = redeem_codes.get_mut(&redeem_code.promotion) {
		// Update Code
		redeem_code.remaining_uses += 1;
	} else {
		let sub = redeem_code
			.entitlements
			.iter()
			.find(|e| matches!(e.data, EntitlementData::Subscription {}));
		let Some(paint) = redeem_code.entitlements.iter().find_map(|e| match e.data {
			EntitlementData::Paint { ref_id, .. } => Some(PaintId::from(ref_id)),
			_ => None,
		}) else {
			return outcome.with_error(error::Error::RedeemCode(format!(
				"{} has no paint entitlement",
				redeem_code.promotion
			)));
		};

		// Unique Code
		redeem_codes.insert(
			redeem_code.promotion.clone(),
			RedeemCode {
				id: RedeemCodeId::new(),
				name: "Legacy Promotion".to_string(),
				description: None,
				tags: vec!["legacy".to_string()],
				code: redeem_code.promotion,
				remaining_uses: 1,
				active_period: None,
				subscription_effect: sub.map(|sub| RedeemCodeSubscriptionEffect {
					id: SubscriptionProductId::from_str(NEW_SUBSCRIPTION_PRODUCT_ID).unwrap(),
					trial_days: sub.app.state.as_ref().and_then(|s| s.trial_duration).map(|d| d as i32),
				}),
				created_by: UserId::nil(),
				effect: paint_events
					.get(&paint)
					.map(|event| CodeEffect::SpecialEvent {
						special_event_id: *event,
					})
					.unwrap_or_else(|| CodeEffect::DirectEntitlement {
						entitlements: vec![EntitlementEdgeKind::Paint { paint_id: paint }],
					}),
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			},
		);
	}

	outcome
}
