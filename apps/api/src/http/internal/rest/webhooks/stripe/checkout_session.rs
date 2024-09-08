use std::str::FromStr;
use std::sync::Arc;

use shared::database::product::codes::{RedeemCode, RedeemCodeId};
use shared::database::product::subscription::{ProviderSubscriptionId, SubscriptionId, SubscriptionPeriod};
use shared::database::queries::{filter, update};

use crate::global::Global;
use crate::http::egvault::metadata::{CheckoutSessionMetadata, CustomerMetadata, StripeMetadata};
use crate::http::egvault::redeem::grant_entitlements;
use crate::http::error::ApiError;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn completed(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	session: stripe::CheckoutSession,
) -> TransactionResult<Option<SubscriptionId>, ApiError> {
	let metadata = session
		.metadata
		.as_ref()
		.map(CheckoutSessionMetadata::from_stripe)
		.transpose()
		.map_err(|err| {
			tracing::error!(error = %err, "failed to deserialize metadata");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?
		.ok_or_else(|| {
			tracing::error!("missing metadata");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;

	match (session.mode, metadata) {
		(stripe::CheckoutSessionMode::Setup, CheckoutSessionMetadata::Setup) => {
			// setup session
			// the customer successfully setup a new payment method, now set it as the
			// default payment method

			let setup_intent = session
				.setup_intent
				.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?
				.id();
			let setup_intent = stripe::SetupIntent::retrieve(&global.stripe_client, &setup_intent, &[])
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to retrieve setup intent");
					TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
				})?;

			let customer_id = session.customer.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?.id();
			let Some(payment_method) = setup_intent.payment_method.map(|p| p.id()) else {
				return Ok(None);
			};

			let customer = stripe::Customer::update(
				&global.stripe_client,
				&customer_id,
				stripe::UpdateCustomer {
					invoice_settings: Some(stripe::CustomerInvoiceSettings {
						default_payment_method: Some(payment_method.to_string()),
						..Default::default()
					}),
					..Default::default()
				},
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update customer");
				TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
			})?;

			let metadata = CustomerMetadata::from_stripe(&customer.metadata.unwrap_or_default()).map_err(|e| {
				tracing::error!(error = %e, "failed to deserialize metadata");
				TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
			})?;

			if let Some(ProviderSubscriptionId::Stripe(sub_id)) = tx
				.find_one(
					filter::filter! {
						SubscriptionPeriod {
							#[query(flatten)]
							subscription_id: SubscriptionId {
								user_id: metadata.user_id,
							},
							#[query(selector = "lt")]
							start: chrono::Utc::now(),
							#[query(selector = "gt")]
							end: chrono::Utc::now(),
						}
					},
					None,
				)
				.await?
				.and_then(|p| p.provider_id)
			{
				stripe::Subscription::update(
					&global.stripe_client,
					&sub_id,
					stripe::UpdateSubscription {
						default_payment_method: Some(&payment_method),
						..Default::default()
					},
				)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to update subscription");
					TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
				})?;
			}
		}
		(stripe::CheckoutSessionMode::Payment, CheckoutSessionMetadata::Redeem { user_id, redeem_code_id }) => {
			// redeem code session
			// the customer successfully redeemed the subscription linked to the redeem
			// code, now we grant access to the entitlements linked to the redeem code
			let redeem_code = global
				.redeem_code_by_id_loader
				.load(redeem_code_id)
				.await
				.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
				.ok_or(TransactionError::custom(ApiError::NOT_FOUND))?;

			grant_entitlements(&mut tx, &redeem_code, user_id).await?;
		}
		_ => {}
	}

	Ok(None)
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

	// session expired so we can increase the remaining uses of the redeem code
	// again
	if metadata.get("IS_REDEEM").is_some_and(|v| v == "true") {
		let redeem_code_id = metadata
			.get("REDEEM_CODE_ID")
			.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		let redeem_code_id = RedeemCodeId::from_str(redeem_code_id).map_err(|e| {
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
