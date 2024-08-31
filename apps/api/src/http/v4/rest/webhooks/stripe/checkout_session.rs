use std::{str::FromStr, sync::Arc};

use shared::database::{
	entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy},
	product::{
		codes::{CodeEffect, RedeemCode, RedeemCodeId},
		subscription::{ProviderSubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy, SubscriptionPeriodId},
		ProductId,
	},
	queries::{filter, update},
	user::UserId,
};

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{TransactionError, TransactionResult, TransactionSession},
};

pub async fn completed(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	session: stripe::CheckoutSession,
) -> TransactionResult<(), ApiError> {
	let Some(metadata) = session.metadata else {
		// ignore metadata sessions that don't have metadata
		return Ok(());
	};

	if session.mode == stripe::CheckoutSessionMode::Setup {
		// setup session
		// the customer successfully setup a new payment method, now set it as the default payment method

		let setup_intent = session.setup_intent.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?.id();
		let setup_intent = stripe::SetupIntent::retrieve(&global.stripe_client, &setup_intent, &[]).await.map_err(|e| {
			tracing::error!(error = %e, "failed to retrieve setup intent");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;

		let customer_id = session.customer.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?.id();
		let Some(payment_method) = setup_intent.payment_method.map(|p| p.id()) else {
			return Ok(());
		};

		let customer = stripe::Customer::update(&global.stripe_client, &customer_id, stripe::UpdateCustomer {
			invoice_settings: Some(stripe::CustomerInvoiceSettings {
				default_payment_method: Some(payment_method.to_string()),
				..Default::default()
			}),
			..Default::default()
		}).await.map_err(|e| {
			tracing::error!(error = %e, "failed to update customer");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;

		if let Some(user_id) = customer.metadata.and_then(|m| m.get("USER_ID").and_then(|i| UserId::from_str(i).ok())) {
			if let Some(ProviderSubscriptionId::Stripe(sub_id)) = tx.find_one(filter::filter! {
				SubscriptionPeriod {
					user_id,
					#[query(selector = "lt")]
					start: chrono::Utc::now(),
					#[query(selector = "gt")]
					end: chrono::Utc::now(),
				}
			}, None).await?.and_then(|p| p.subscription_id) {
				stripe::Subscription::update(&global.stripe_client, &sub_id, stripe::UpdateSubscription {
					default_payment_method: Some(&payment_method),
					..Default::default()
				}).await.map_err(|e| {
					tracing::error!(error = %e, "failed to update subscription");
					TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
				})?;
			}
		}
	} else if metadata.get("IS_REDEEM").is_some_and(|v| v == "true") {
		// redeem code session
		// the customer successfully redeemed the subscription linked to the redeem code, now we grant access to the entitlements linked to the redeem code

		let redeem_code_id = metadata
			.get("REDEEM_CODE_ID")
			.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		let redeem_code_id = RedeemCodeId::from_str(&redeem_code_id).map_err(|e| {
			tracing::error!(error = %e, "invalid redeem code id");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;

		let user_id = metadata
			.get("USER_ID")
			.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		let user_id = UserId::from_str(&user_id).map_err(|e| {
			tracing::error!(error = %e, "invalid user id");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;

		let redeem_code = tx
			.find_one(
				filter::filter! {
					RedeemCode {
						#[query(rename = "_id")]
						id: redeem_code_id,
					}
				},
				None,
			)
			.await?
			.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		for effect in redeem_code.effects.into_iter().filter_map(|e| match e {
			CodeEffect::Entitlement { edge } => Some(edge),
			_ => None,
		}) {
			tx.insert_one(
				EntitlementEdge {
					id: EntitlementEdgeId {
						from: EntitlementEdgeKind::User { user_id },
						to: effect,
						managed_by: Some(EntitlementEdgeManagedBy::RedeemCode {
							redeem_code_id: redeem_code.id,
						}),
					},
				},
				None,
			)
			.await?;
		}
	} else if metadata.get("IS_GIFT").is_some_and(|v| v == "true") {
		// gift code session
		// the gift sub payment was successful, now we add one subscription period for the recipient

		let Some(payment_id) = session.payment_intent.map(|p| p.id()) else {
			// ignore non-payment sessions
			return Ok(());
		};

		let period_duration: u32 = metadata
			.get("PERIOD_DURATION_MONTHS")
			.and_then(|v| v.parse().ok())
			.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		let receiving_user = metadata
			.get("USER_ID")
			.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		let receiving_user = UserId::from_str(&receiving_user).map_err(|e| {
			tracing::error!(error = %e, "invalid user id");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;

		let paying_user = metadata
			.get("CUSTOMER_ID")
			.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		let paying_user = UserId::from_str(&paying_user).map_err(|e| {
			tracing::error!(error = %e, "invalid user id");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;

		let product_id = metadata
			.get("PRODUCT_ID")
			.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;
		let product_id = ProductId::from_str(&product_id).map_err(|e| {
			tracing::error!(error = %e, "invalid product id");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;

		let start =
			chrono::DateTime::from_timestamp(session.created, 0).ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

		let end = start
			.checked_add_months(chrono::Months::new(period_duration)) // It's fine to use this function here since UTC doens't have daylight saving time transitions
			.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		tx.insert_one(
			SubscriptionPeriod {
				id: SubscriptionPeriodId::new(),
				subscription_id: None,
				user_id: receiving_user,
				start,
				end,
				is_trial: false,
				created_by: SubscriptionPeriodCreatedBy::Gift {
					gifter: paying_user,
					payment: payment_id.into(),
				},
				product_ids: vec![product_id],
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			},
			None,
		)
		.await?;
	}

	Ok(())
}

pub async fn expired(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	session: stripe::CheckoutSession,
) -> TransactionResult<(), ApiError> {
	let Some(metadata) = session.metadata else {
		// ignore metadata sessions that don't have metadata
		return Ok(());
	};

	// session expired so we can increase the remaining uses of the redeem code again
	if metadata.get("IS_REDEEM").is_some_and(|v| v == "true") {
		let redeem_code_id = metadata
			.get("REDEEM_CODE_ID")
			.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		let redeem_code_id = RedeemCodeId::from_str(&redeem_code_id).map_err(|e| {
			tracing::error!(error = %e, "invalid redeem code id");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;

		tx.update_one(
			filter::filter! {
				RedeemCode {
					#[query(rename = "_id")]
					id: redeem_code_id,
				}
			},
			update::update! {
				#[query(inc)]
				RedeemCode {
					remaining_uses: 1,
				},
			},
			None,
		)
		.await?;
	}

	Ok(())
}
