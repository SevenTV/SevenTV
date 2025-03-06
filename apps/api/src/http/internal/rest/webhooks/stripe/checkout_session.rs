use std::ops::Deref;
use std::sync::Arc;

use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::product::codes::RedeemCode;
use shared::database::product::subscription::{ProviderSubscriptionId, SubscriptionId, SubscriptionPeriod};
use shared::database::queries::{filter, update};
use shared::database::MongoCollection;
use tracing::Instrument;

use crate::global::Global;
use crate::http::egvault::metadata::{CheckoutSessionMetadata, CustomerMetadata, StripeMetadata};
use crate::http::egvault::redeem::grant_entitlements;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::stripe_client::SafeStripeClient;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StripeRequest {
	RetrieveSetupIntent,
	UpdateCustomer,
	SubscriptionUpdate,
}

impl std::fmt::Display for StripeRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::RetrieveSetupIntent => write!(f, "retrieve_setup_intent"),
			Self::UpdateCustomer => write!(f, "update_customer"),
			Self::SubscriptionUpdate => write!(f, "subscription_update"),
		}
	}
}

#[tracing::instrument(skip_all, name = "stripe::checkout_session::completed")]
pub async fn completed(
	global: &Arc<Global>,
	stripe_client: SafeStripeClient<super::StripeRequest>,
	mut tx: TransactionSession<'_, ApiError>,
	session: stripe::CheckoutSession,
) -> TransactionResult<Option<SubscriptionId>, ApiError> {
	let metadata = session
		.metadata
		.as_ref()
		.filter(|m| !m.is_empty())
		.map(CheckoutSessionMetadata::from_stripe)
		.transpose()
		.map_err(|err| {
			tracing::error!(error = %err, "failed to deserialize metadata");
			TransactionError::Custom(ApiError::internal_server_error(
				ApiErrorCode::StripeError,
				"failed to deserialize metadata",
			))
		})?
		.ok_or_else(|| {
			tracing::error!("missing metadata");
			TransactionError::Custom(ApiError::internal_server_error(ApiErrorCode::StripeError, "missing metadata"))
		})?;

	match (session.mode, metadata) {
		(stripe::CheckoutSessionMode::Setup, CheckoutSessionMetadata::Setup) => {
			// setup session
			// the customer successfully setup a new payment method, now set it as the
			// default payment method

			let setup_intent = session
				.setup_intent
				.ok_or_else(|| {
					TransactionError::Custom(ApiError::bad_request(ApiErrorCode::StripeError, "missing setup intent"))
				})?
				.id();

			let setup_intent = stripe::SetupIntent::retrieve(
				stripe_client
					.client(super::StripeRequest::CheckoutSession(StripeRequest::RetrieveSetupIntent))
					.await
					.deref(),
				&setup_intent,
				&[],
			)
			.instrument(tracing::info_span!("setup_intent_retrieve"))
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to retrieve setup intent");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to retrieve setup intent",
				))
			})?;

			let customer_id = session
				.customer
				.ok_or_else(|| {
					TransactionError::Custom(ApiError::bad_request(ApiErrorCode::StripeError, "missing customer"))
				})?
				.id();
			let Some(payment_method) = setup_intent.payment_method.map(|p| p.id()) else {
				return Ok(None);
			};

			let customer = stripe::Customer::update(
				stripe_client
					.client(super::StripeRequest::CheckoutSession(StripeRequest::UpdateCustomer))
					.await
					.deref(),
				&customer_id,
				stripe::UpdateCustomer {
					invoice_settings: Some(stripe::CustomerInvoiceSettings {
						default_payment_method: Some(payment_method.to_string()),
						..Default::default()
					}),
					..Default::default()
				},
			)
			.instrument(tracing::info_span!("customer_update"))
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update customer");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to update customer",
				))
			})?;

			let metadata = CustomerMetadata::from_stripe(&customer.metadata.unwrap_or_default()).map_err(|e| {
				tracing::error!(error = %e, "failed to deserialize metadata");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to deserialize metadata",
				))
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
					stripe_client
						.client(super::StripeRequest::CheckoutSession(StripeRequest::SubscriptionUpdate))
						.await
						.deref(),
					&sub_id,
					stripe::UpdateSubscription {
						default_payment_method: Some(&payment_method),
						..Default::default()
					},
				)
				.instrument(tracing::info_span!("subscription_update"))
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to update subscription");
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::StripeError,
						"failed to update subscription",
					))
				})?;
			}
		}
		(_, CheckoutSessionMetadata::Redeem { user_id, redeem_code_id }) => {
			// redeem code session
			// the customer successfully redeemed the subscription linked to the redeem
			// code, now we grant access to the entitlements linked to the redeem code
			let redeem_code = global
				.redeem_code_by_id_loader
				.load(redeem_code_id)
				.await
				.map_err(|_| {
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::StripeError,
						"failed to load redeem code",
					))
				})?
				.ok_or_else(|| {
					TransactionError::Custom(ApiError::not_found(ApiErrorCode::StripeError, "redeem code not found"))
				})?;

			grant_entitlements(global, &redeem_code, user_id)
				.await
				.map_err(TransactionError::Custom)?;
		}
		(_, CheckoutSessionMetadata::Pickems { user_id, product_id }) => {
			EntitlementEdge::collection(&global.db)
				.insert_one(EntitlementEdge {
					id: EntitlementEdgeId {
						from: EntitlementEdgeKind::User { user_id },
						to: EntitlementEdgeKind::Product { product_id },
						managed_by: None,
					},
				})
				.await
				.map_err(|_| {
					TransactionError::Custom(ApiError::not_found(
						ApiErrorCode::StripeError,
						"unable to create entitlement edge",
					))
				})?;
		}
		_ => {}
	}

	Ok(None)
}

#[tracing::instrument(skip_all, name = "stripe::checkout_session::expired")]
pub async fn expired(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	session: stripe::CheckoutSession,
) -> TransactionResult<(), ApiError> {
	let metadata = session
		.metadata
		.as_ref()
		.filter(|m| !m.is_empty())
		.map(CheckoutSessionMetadata::from_stripe)
		.transpose()
		.map_err(|err| {
			tracing::error!(error = %err, "failed to deserialize metadata");
			TransactionError::Custom(ApiError::internal_server_error(
				ApiErrorCode::StripeError,
				"failed to deserialize metadata",
			))
		})?
		.ok_or_else(|| {
			tracing::error!("missing metadata");
			TransactionError::Custom(ApiError::internal_server_error(ApiErrorCode::StripeError, "missing metadata"))
		})?;

	if let CheckoutSessionMetadata::Redeem { redeem_code_id, .. } = metadata {
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
				#[query(set)]
				RedeemCode {
					updated_at: chrono::Utc::now(),
					search_updated_at: &None,
				},
			},
			None,
		)
		.await?;
	}

	Ok(())
}
