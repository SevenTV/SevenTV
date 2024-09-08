use std::ops::Deref;
use std::sync::Arc;

use axum::extract::State;
use axum::{Extension, Json};
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy};
use shared::database::product::codes::{CodeEffect, RedeemCode};
use shared::database::product::subscription::{SubscriptionId, SubscriptionPeriod};
use shared::database::product::TimePeriod;
use shared::database::queries::{filter, update};
use shared::database::user::UserId;

use super::metadata::{CheckoutSessionMetadata, StripeMetadata, SubscriptionMetadata};
use super::{create_checkout_session_params, find_or_create_customer};
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::transactions::{with_transaction, TransactionError, TransactionResult, TransactionSession};

pub async fn grant_entitlements(
	tx: &mut TransactionSession<'_, ApiError>,
	redeem_code: &RedeemCode,
	user_id: UserId,
) -> TransactionResult<(), ApiError> {
	for (to, extends_subscription) in redeem_code.effects.iter().filter_map(|e| match e {
		CodeEffect::Entitlement {
			edge,
			extends_subscription,
		} => Some((edge, extends_subscription)),
		_ => None,
	}) {
		let from = match extends_subscription {
			Some(extends_subscription) => EntitlementEdgeKind::Subscription {
				subscription_id: SubscriptionId {
					user_id,
					product_id: *extends_subscription,
				},
			},
			None => EntitlementEdgeKind::User { user_id },
		};

		tx.insert_one(
			EntitlementEdge {
				id: EntitlementEdgeId {
					from,
					to: to.clone(),
					managed_by: Some(EntitlementEdgeManagedBy::RedeemCode {
						redeem_code_id: redeem_code.id,
					}),
				},
			},
			None,
		)
		.await?;
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
	/// list of ids of cosmetics that the user received
	items: Vec<String>,
}

pub async fn redeem(
	State(global): State<Arc<Global>>,
	Extension(ip): Extension<std::net::IpAddr>,
	auth_session: Option<AuthSession>,
	Json(body): Json<RedeemRequest>,
) -> Result<Json<RedeemResponse>, ApiError> {
	let auth_session = auth_session.ok_or(ApiError::UNAUTHORIZED)?;

	let stripe_client = global.stripe_client.safe().await;

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
					},
					None,
				)
				.await?
				.ok_or(TransactionError::custom(ApiError::NOT_FOUND))?;

			let is_subscribed = tx
				.find_one(
					filter::filter! {
						SubscriptionPeriod {
							#[query(flatten)]
							subscription_id: SubscriptionId {
								user_id: auth_session.user_id(),
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
				.is_some();

			if let Some((product_id, trial_days, false)) = code.effects.iter().find_map(|e| match e {
				CodeEffect::SubscriptionProduct { id, trial_days } => Some((id, trial_days, is_subscribed)),
				_ => None,
			}) {
				// the user is not subscribed and the effects contain a subscription product

				let customer_id = match auth_session
					.user(&global)
					.await
					.map_err(TransactionError::custom)?
					.stripe_customer_id
					.clone()
				{
					Some(id) => id,
					None => find_or_create_customer(&global, stripe_client.client(0).await, auth_session.user_id(), None)
						.await
						.map_err(TransactionError::custom)?,
				};

				let product = global
					.subscription_product_by_id_loader
					.load(*product_id)
					.await
					.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
					.ok_or_else(|| {
						tracing::warn!(
							"could not find subscription product for redeem code: {} product id: {product_id}",
							code.id
						);
						TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
					})?;

				let variant = product.variants.get(product.default_variant_idx as usize).ok_or_else(|| {
					tracing::warn!(
						"could not find default variant for subscription product for redeem code: {} product id: {product_id}",
						code.id
					);
					TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
				})?;

				let mut params = create_checkout_session_params(
					&global,
					ip,
					customer_id,
					&variant.id,
					product.default_currency,
					&variant.currency_prices,
				)
				.await;

				params.mode = Some(stripe::CheckoutSessionMode::Subscription);

				params.subscription_data = Some(stripe::CreateCheckoutSessionSubscriptionData {
					metadata: Some(SubscriptionMetadata {
						user_id: auth_session.user_id(),
						customer_id: None,
					}.to_stripe()),
					trial_period_days: Some(*trial_days),
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
						user_id: auth_session.user_id(),
					}
					.to_stripe(),
				);

				let url = stripe::CheckoutSession::create(stripe_client.client(1).await.deref(), params)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to create checkout session");
						TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
					})?
					.url
					.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

				Ok(RedeemResponse {
					authorize_url: Some(url),
					items: vec![],
				})
			} else {
				// the effects contain no subscription products

				let items = code
					.effects
					.iter()
					.filter_map(|e| match e {
						CodeEffect::Entitlement {
							edge: EntitlementEdgeKind::Badge { badge_id },
							..
						} => Some(badge_id.to_string()),
						CodeEffect::Entitlement {
							edge: EntitlementEdgeKind::Paint { paint_id },
							..
						} => Some(paint_id.to_string()),
						_ => None,
					})
					.collect();

				grant_entitlements(&mut tx, &code, auth_session.user_id()).await?;

				Ok(RedeemResponse {
					authorize_url: None,
					items,
				})
			}
		}
	})
	.await;

	match res {
		Ok(res) => Ok(Json(res)),
		Err(TransactionError::Custom(e)) => Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			Err(ApiError::INTERNAL_SERVER_ERROR)
		}
	}
}
