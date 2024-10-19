use std::ops::Deref;
use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy};
use shared::database::product::codes::{CodeEffect, RedeemCode, RedeemCodeSubscriptionEffect};
use shared::database::product::subscription::SubscriptionId;
use shared::database::product::TimePeriod;
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, UserPermission};
use shared::database::user::UserId;
use shared::database::Id;

use super::metadata::{CheckoutSessionMetadata, StripeMetadata, SubscriptionMetadata};
use super::{create_checkout_session_params, find_or_create_customer, CheckoutProduct};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::ratelimit::RateLimitRequest;
use crate::transactions::{with_transaction, TransactionError, TransactionResult, TransactionSession};

pub async fn grant_entitlements(
	tx: &mut TransactionSession<'_, ApiError>,
	redeem_code: &RedeemCode,
	user_id: UserId,
) -> TransactionResult<(), ApiError> {
	// If the redeem code has a subscription effect then all entitlements should be
	// attached to the subscription, not the user.
	let from = match &redeem_code.subscription_effect {
		Some(subscription_effect) => EntitlementEdgeKind::Subscription {
			subscription_id: SubscriptionId {
				user_id,
				product_id: subscription_effect.id,
			},
		},
		None => EntitlementEdgeKind::User { user_id },
	};

	match &redeem_code.effect {
		CodeEffect::DirectEntitlement { entitlements } => {
			tx.insert_many(
				entitlements.iter().map(|e| EntitlementEdge {
					id: EntitlementEdgeId {
						from: from.clone(),
						to: e.clone(),
						managed_by: Some(EntitlementEdgeManagedBy::RedeemCode {
							redeem_code_id: redeem_code.id,
						}),
					},
				}),
				None,
			)
			.await?;
		}
		CodeEffect::SpecialEvent { special_event_id } => {
			tx.insert_one(
				EntitlementEdge {
					id: EntitlementEdgeId {
						from,
						to: EntitlementEdgeKind::SpecialEvent {
							special_event_id: *special_event_id,
						},
						managed_by: Some(EntitlementEdgeManagedBy::RedeemCode {
							redeem_code_id: redeem_code.id,
						}),
					},
				},
				None,
			)
			.await?;
		}
	}

	Ok(())
}

#[derive(Debug, serde::Deserialize)]
pub struct RedeemRequest {
	code: String,
}

#[derive(Debug, serde::Serialize)]
pub struct RedeemResponse {
	/// Url that the website will open
	authorize_url: Option<String>,
}

#[derive(Debug, Clone)]
enum StripeRequest {
	CreateCustomer,
	CreateCheckoutSession,
}

impl std::fmt::Display for StripeRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::CreateCustomer => write!(f, "create_customer"),
			Self::CreateCheckoutSession => write!(f, "create_checkout_session"),
		}
	}
}

pub async fn redeem(
	State(global): State<Arc<Global>>,
	Extension(session): Extension<Session>,
	Json(body): Json<RedeemRequest>,
) -> Result<impl IntoResponse, ApiError> {
	let authed_user = session.user()?;
	let req = RateLimitRequest::new(RateLimitResource::EgVaultRedeem, &session);

	if !authed_user.has(UserPermission::Billing) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"this user isn't allowed to use billing features",
		));
	}

	req.http(&global, async {
		let session = &session;

		let stripe_client = global.stripe_client.safe(Id::<()>::new()).await;

		let res = with_transaction(&global, |mut tx| {
			let global = Arc::clone(&global);

			async move {
				let code = tx
					.find_one_and_update(
						filter::filter! {
							RedeemCode {
								code: body.code,
								#[query(selector = "gt")]
								remaining_uses: 0,
								#[query(flatten)]
								active_period: TimePeriod {
									#[query(selector = "lt")]
									start: chrono::Utc::now(),
									#[query(selector = "gt")]
									end: chrono::Utc::now(),
								},
							}
						},
						update::update! {
							#[query(inc)]
							RedeemCode {
								remaining_uses: -1,
							},
							#[query(set)]
							RedeemCode {
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							},
						},
						None,
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "redeem code not found"))
					})?;

				let not_subscribed = global
					.active_subscription_period_by_user_id_loader
					.load(authed_user.id)
					.await
					.map_err(|()| {
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::LoadError,
							"failed to load subscription period",
						))
					})?
					.is_none();

				// If the user is not subscribed and the redeem code has a subscription effect
				// which grants a trial period, then we should start their trial period.
				if let Some(RedeemCodeSubscriptionEffect {
					id: product_id,
					trial_days: Some(trial_days),
				}) = not_subscribed.then_some(code.subscription_effect.as_ref()).flatten()
				{
					// the user is not subscribed and the effects contain a subscription product
					let customer_id = match authed_user.stripe_customer_id.clone() {
						Some(id) => id,
						None => find_or_create_customer(
							&global,
							stripe_client.client(StripeRequest::CreateCustomer).await,
							authed_user.id,
							None,
						)
						.await
						.map_err(TransactionError::Custom)?,
					};

					let product = global
						.subscription_product_by_id_loader
						.load(*product_id)
						.await
						.map_err(|_| {
							TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::LoadError,
								"failed to load subscription product",
							))
						})?
						.ok_or_else(|| {
							tracing::warn!(
								"could not find subscription product for redeem code: {} product id: {product_id}",
								code.id
							);
							TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::LoadError,
								"failed to load subscription product",
							))
						})?;

					let variant = product.variants.get(product.default_variant_idx as usize).ok_or_else(|| {
						tracing::warn!(
							"could not find default variant for subscription product for redeem code: {} product id: {product_id}",
							code.id
						);
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::LoadError,
							"failed to load subscription product",
						))
					})?;

					let mut params = create_checkout_session_params(
						&global,
						session.ip(),
						customer_id,
						CheckoutProduct::Price(variant.id.0.clone()),
						product.default_currency,
						&variant.currency_prices,
					)
					.await;

					params.mode = Some(stripe::CheckoutSessionMode::Subscription);

					params.subscription_data = Some(stripe::CreateCheckoutSessionSubscriptionData {
						metadata: Some(SubscriptionMetadata {
							user_id: authed_user.id,
							customer_id: None,
						}.to_stripe()),
						trial_period_days: Some(*trial_days as u32),
						trial_settings: Some(stripe::CreateCheckoutSessionSubscriptionDataTrialSettings {
							end_behavior: stripe::CreateCheckoutSessionSubscriptionDataTrialSettingsEndBehavior {
								missing_payment_method:
									stripe::CreateCheckoutSessionSubscriptionDataTrialSettingsEndBehaviorMissingPaymentMethod::Cancel,
							},
						}),
						..Default::default()
					});

					params.metadata = Some(
						CheckoutSessionMetadata::Redeem {
							redeem_code_id: code.id,
							user_id: authed_user.id,
						}
						.to_stripe(),
					);

					let url = stripe::CheckoutSession::create(
						stripe_client.client(StripeRequest::CreateCheckoutSession).await.deref(),
						params,
					)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to create checkout session");
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::StripeError,
							"failed to create checkout session",
						))
					})?
					.url
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::StripeError,
							"failed to create checkout session",
						))
					})?;

					Ok(RedeemResponse {
						authorize_url: Some(url),
					})
				} else {
					// the effects contain no subscription products
					grant_entitlements(&mut tx, &code, authed_user.id).await?;

					Ok(RedeemResponse { authorize_url: None })
				}
			}
		})
		.await;

		match res {
			Ok(res) => Ok(Json(res)),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	})
	.await
}
