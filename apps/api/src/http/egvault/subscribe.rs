use std::ops::Deref;
use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use shared::database::product::subscription::{SubscriptionId, SubscriptionPeriod};
use shared::database::product::{SubscriptionProduct, SubscriptionProductKind, SubscriptionProductVariant};
use shared::database::queries::filter;
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, UserPermission};
use shared::database::user::UserId;
use shared::database::MongoCollection;

use super::metadata::{CheckoutSessionMetadata, InvoiceMetadata, StripeMetadata, SubscriptionMetadata};
use super::{create_checkout_session_params, find_or_create_customer, CheckoutProduct};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::extract::Query;
use crate::http::middleware::session::Session;
use crate::ratelimit::RateLimitRequest;

#[derive(Debug, serde::Deserialize)]
pub struct SubscribeQuery {
	renew_interval: SubscriptionRenewInterval,
	/// only "stripe" allowed
	payment_method: String,
	/// always true
	#[serde(rename = "next")]
	_next: bool,
	gift_for: Option<UserId>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SubscribeBody {
	prefill: Prefill,
}

#[derive(Debug, serde::Deserialize)]
pub struct Prefill {
	pub first_name: String,
	pub last_name: String,
	pub email: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionRenewInterval {
	Monthly,
	Yearly,
}

impl From<SubscriptionRenewInterval> for SubscriptionProductKind {
	fn from(value: SubscriptionRenewInterval) -> Self {
		match value {
			SubscriptionRenewInterval::Monthly => Self::Monthly,
			SubscriptionRenewInterval::Yearly => Self::Yearly,
		}
	}
}

#[derive(Debug, serde::Serialize)]
pub struct SubscribeResponse {
	/// Url that the website will open in a new tab
	url: String,
	/// The user id of the user that receives the subscription
	user_id: UserId,
}

pub async fn subscribe(
	State(global): State<Arc<Global>>,
	Query(query): Query<SubscribeQuery>,
	Extension(session): Extension<Session>,
	Json(body): Json<SubscribeBody>,
) -> Result<impl IntoResponse, ApiError> {
	let authed_user = session.user()?;

	if query.payment_method != "stripe" {
		return Err(ApiError::bad_request(
			ApiErrorCode::BadRequest,
			"payment method not supported",
		));
	}

	if !authed_user.has(UserPermission::Billing) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"this user isn't allowed to use billing features",
		));
	}

	let kind = SubscriptionProductKind::from(query.renew_interval);
	let req = RateLimitRequest::new(RateLimitResource::EgVaultSubscribe, &session);

	req.http(&global, async {
		let product: SubscriptionProduct = SubscriptionProduct::collection(&global.db)
			.find_one(filter::filter! {
				SubscriptionProduct {
					#[query(flatten)]
					variants: SubscriptionProductVariant {
						#[query(serde)]
						kind: &kind,
						active: true,
					}
				}
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to find subscription product");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to find subscription product")
			})?
			.ok_or_else(|| ApiError::internal_server_error(ApiErrorCode::LoadError, "subscription product not found"))?;

		let variant = product.variants.into_iter().find(|v| v.kind == kind && v.active).unwrap();

		let customer_id = match authed_user.stripe_customer_id.clone() {
			Some(id) => id,
			None => {
				// We don't need the safe client here because this won't be retried
				find_or_create_customer(
					&global,
					global.stripe_client.client().await,
					authed_user.id,
					Some(body.prefill),
				)
				.await?
			}
		};

		let mut params = create_checkout_session_params(
			&global,
			session.ip(),
			customer_id,
			match &query.gift_for {
				Some(_) => CheckoutProduct::Gift(product.provider_id),
				None => CheckoutProduct::Price(variant.id.0.clone()),
			},
			product.default_currency,
			&variant.currency_prices,
		)
		.await;

		let receiving_user = if let Some(gift_for) = query.gift_for {
			let receiving_user = global
				.user_loader
				.load_fast(&global, gift_for)
				.await
				.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
				.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

			// TODO: should we dataload this?
			let is_subscribed = SubscriptionPeriod::collection(&global.db)
				.find_one(filter::filter! {
					SubscriptionPeriod {
						#[query(flatten)]
						subscription_id: SubscriptionId {
							user_id: receiving_user.id,
						},
						#[query(selector = "lt")]
						start: chrono::Utc::now(),
						#[query(selector = "gt")]
						end: chrono::Utc::now(),
					}
				})
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to load subscription periods");
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription periods")
				})?
				.is_some();
			if is_subscribed {
				return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "user is already subscribed"));
			}

			params.mode = Some(stripe::CheckoutSessionMode::Payment);
			params.payment_intent_data = Some(stripe::CreateCheckoutSessionPaymentIntentData {
				description: Some(format!(
					"Gift subscription for {} (7TV:{})",
					receiving_user
						.connections
						.first()
						.map(|c| { format!("{} ({}:{})", c.platform_display_name, c.platform, c.platform_id) })
						.unwrap_or_else(|| "Unknown User".to_owned()),
					receiving_user.id
				)),
				..Default::default()
			});

			params.invoice_creation = Some(stripe::CreateCheckoutSessionInvoiceCreation {
				enabled: true,
				invoice_data: Some(stripe::CreateCheckoutSessionInvoiceCreationInvoiceData {
					metadata: Some(
						InvoiceMetadata::Gift {
							customer_id: authed_user.id,
							user_id: receiving_user.id,
							product_id: variant.id.clone(),
							subscription_product_id: Some(product.id),
						}
						.to_stripe(),
					),
					..Default::default()
				}),
			});

			params.metadata = Some(CheckoutSessionMetadata::Gift.to_stripe());

			receiving_user.id
		} else {
			// TODO: should we dataload this?
			let is_subscribed = SubscriptionPeriod::collection(&global.db)
				.find_one(filter::filter! {
					SubscriptionPeriod {
						#[query(flatten)]
						subscription_id: SubscriptionId {
							user_id: authed_user.id,
						},
						#[query(selector = "lt")]
						start: chrono::Utc::now(),
						#[query(selector = "gt")]
						end: chrono::Utc::now(),
					}
				})
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to load subscription periods");
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription periods")
				})?
				.is_some();

			if is_subscribed {
				return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "user is already subscribed"));
			}

			params.mode = Some(stripe::CheckoutSessionMode::Subscription);
			params.subscription_data = Some(stripe::CreateCheckoutSessionSubscriptionData {
				metadata: Some(
					SubscriptionMetadata {
						user_id: authed_user.id,
						customer_id: None,
					}
					.to_stripe(),
				),
				..Default::default()
			});

			params.metadata = Some(CheckoutSessionMetadata::Subscription.to_stripe());

			authed_user.id
		};

		// We don't need the safe client here because this won't be retried
		let session_url = stripe::CheckoutSession::create(global.stripe_client.client().await.deref(), params)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to create checkout session");
				ApiError::internal_server_error(ApiErrorCode::StripeError, "failed to create checkout session")
			})?
			.url
			.ok_or_else(|| {
				ApiError::internal_server_error(ApiErrorCode::StripeError, "failed to create checkout session")
			})?;

		Ok(Json(SubscribeResponse {
			url: session_url,
			user_id: receiving_user,
		}))
	})
	.await
}
