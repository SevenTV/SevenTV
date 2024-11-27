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
use shared::database::{Id, MongoCollection};

use super::metadata::{CheckoutSessionMetadata, StripeMetadata, SubscriptionMetadata};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::ratelimit::RateLimitRequest;
use crate::stripe_common::{create_checkout_session_params, find_or_create_customer, CheckoutProduct};

pub async fn grant_entitlements(
	global: &Arc<Global>,
	// tx: &mut TransactionSession<'_, ApiError>,
	redeem_code: &RedeemCode,
	user_id: UserId,
) -> Result<(), ApiError> {
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

	EntitlementEdge::collection(&global.db)
		.delete_many(filter::filter! {
			EntitlementEdge {
				#[query(flatten, rename = "_id")]
				id: EntitlementEdgeId {
					#[query(serde)]
					managed_by: Some(EntitlementEdgeManagedBy::RedeemCode {
						redeem_code_id: redeem_code.id,
					}),
				},
			}
		})
		.await
		.map_err(|err| {
			tracing::error!(error = %err, "failed to delete existing entitlements");
			ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to delete existing entitlements")
		})?;

	match &redeem_code.effect {
		CodeEffect::DirectEntitlement { entitlements } => {
			EntitlementEdge::collection(&global.db)
				.insert_many(entitlements.iter().map(|e| EntitlementEdge {
					id: EntitlementEdgeId {
						from: from.clone(),
						to: e.clone(),
						managed_by: Some(EntitlementEdgeManagedBy::RedeemCode {
							redeem_code_id: redeem_code.id,
						}),
					},
				}))
				.await
				.map_err(|err| {
					tracing::error!(error = %err, "failed to insert entitlements");
					ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to insert entitlements")
				})?;
		}
		CodeEffect::SpecialEvent { special_event_id } => {
			EntitlementEdge::collection(&global.db)
				.insert_one(EntitlementEdge {
					id: EntitlementEdgeId {
						from,
						to: EntitlementEdgeKind::SpecialEvent {
							special_event_id: *special_event_id,
						},
						managed_by: Some(EntitlementEdgeManagedBy::RedeemCode {
							redeem_code_id: redeem_code.id,
						}),
					},
				})
				.await
				.map_err(|err| {
					tracing::error!(error = %err, "failed to insert entitlements");
					ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to insert entitlements")
				})?;
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

	if body.code.len() > 24 {
		return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "redeem code is too long"));
	}

	req.http(&global, async {
		let session = &session;

		let stripe_client = global.stripe_client.safe(Id::<()>::new()).await;
		let global = &global;

		// let res = transaction_with_mutex(global,
		// Some(EgVaultMutexKey::RedeemCode(body.code.clone()).into()), |mut tx| async
		// move {
		let code = RedeemCode::collection(&global.db)
			.find_one_and_update(
				filter::Filter::and([
					filter::filter! {
						RedeemCode {
							code: body.code,
							#[query(selector = "gt")]
							remaining_uses: 0,
						}
					}
					.into(),
					filter::Filter::or([
						filter::filter! {
							RedeemCode {
								#[query(flatten)]
								active_period: TimePeriod {
									#[query(selector = "lt")]
									start: chrono::Utc::now(),
									#[query(selector = "gt")]
									end: chrono::Utc::now(),
								},
							}
						},
						filter::filter! {
							RedeemCode {
								#[query(serde)]
								active_period: &None,
							}
						},
					]),
				]),
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
			)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to load redeem code");

				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load redeem code")
			})?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::BadRequest, "redeem code not found"))?;

		let not_subscribed = global
			.active_subscription_period_by_user_id_loader
			.load(authed_user.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription period"))?
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
				None => {
					find_or_create_customer(
						global,
						stripe_client.client(StripeRequest::CreateCustomer).await,
						authed_user.id,
						None,
					)
					.await?
				}
			};

			let product = global
				.subscription_product_by_id_loader
				.load(*product_id)
				.await
				.map_err(|_| {
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription product")
				})?
				.ok_or_else(|| {
					tracing::warn!(
						"could not find subscription product for redeem code: {} product id: {product_id}",
						code.id
					);
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription product")
				})?;

			let variant = product.variants.get(product.default_variant_idx as usize).ok_or_else(|| {
				tracing::warn!(
					"could not find default variant for subscription product for redeem code: {} product id: {product_id}",
					code.id
				);
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription product")
			})?;

			let success_url = global
				.config
				.api
				.website_origin
				.join("/subscribe/complete?with_provider=stripe")
				.unwrap()
				.to_string();
			let cancel_url = global
				.config
				.api
				.website_origin
				.join("/subscribe/cancel?with_provider=stripe")
				.unwrap()
				.to_string();

			let mut params = create_checkout_session_params(
				global,
				session.ip(),
				customer_id,
				CheckoutProduct::Price(variant.id.0.clone()),
				product.default_currency,
				&variant.currency_prices,
				&success_url,
				&cancel_url,
			)
			.await;

			params.mode = Some(stripe::CheckoutSessionMode::Subscription);

			params.subscription_data = Some(stripe::CreateCheckoutSessionSubscriptionData {
				metadata: Some(
					SubscriptionMetadata {
						user_id: authed_user.id,
						customer_id: None,
					}
					.to_stripe(),
				),
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
				ApiError::internal_server_error(ApiErrorCode::StripeError, "failed to create checkout session")
			})?
			.url
			.ok_or_else(|| {
				ApiError::internal_server_error(ApiErrorCode::StripeError, "failed to create checkout session")
			})?;

			Ok::<_, ApiError>(Json(RedeemResponse {
				authorize_url: Some(url),
			}))
		} else {
			// the effects contain no subscription products
			grant_entitlements(global, &code, authed_user.id).await?;

			Ok::<_, ApiError>(Json(RedeemResponse { authorize_url: None }))
		}
	})
	.await
}
