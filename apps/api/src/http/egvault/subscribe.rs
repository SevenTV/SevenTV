use std::ops::Deref;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use shared::database::product::subscription::{SubscriptionId, SubscriptionPeriod};
use shared::database::product::{SubscriptionProduct, SubscriptionProductKind, SubscriptionProductVariant};
use shared::database::queries::filter;
use shared::database::user::UserId;
use shared::database::MongoCollection;

use super::metadata::{CheckoutSessionMetadata, InvoiceMetadata, StripeMetadata, SubscriptionMetadata};
use super::{create_checkout_session_params, find_or_create_customer};
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::extract::Query;
use crate::http::middleware::auth::AuthSession;

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
	Extension(ip): Extension<std::net::IpAddr>,
	auth_session: Option<AuthSession>,
	Json(body): Json<SubscribeBody>,
) -> Result<Json<SubscribeResponse>, ApiError> {
	let auth_session = auth_session.ok_or(ApiError::UNAUTHORIZED)?;

	if query.payment_method != "stripe" {
		return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "payment method not supported"));
	}

	let kind = SubscriptionProductKind::from(query.renew_interval);

	let product: SubscriptionProduct = SubscriptionProduct::collection(&global.db)
		.find_one(filter::filter! {
			SubscriptionProduct {
				#[query(flatten)]
				variants: SubscriptionProductVariant {
					#[query(serde)]
					kind: &kind,
					active: true,
					gift: query.gift_for.is_some(),
				}
			}
		})
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to find subscription product");
			ApiError::INTERNAL_SERVER_ERROR
		})?
		.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "subscription product not found"))?;

	let variant = product
		.variants
		.into_iter()
		.find(|v| v.kind == kind && v.gift == query.gift_for.is_some())
		.unwrap();

	let customer_id = match auth_session.user(&global).await?.stripe_customer_id.clone() {
		Some(id) => id,
		None => {
			// We don't need the safe client here because this won't be retried
			find_or_create_customer(
				&global,
				global.stripe_client.client().await,
				auth_session.user_id(),
				Some(body.prefill),
			)
			.await?
		}
	};

	let mut params = create_checkout_session_params(
		&global,
		ip,
		customer_id,
		&variant.id,
		product.default_currency,
		&variant.currency_prices,
	)
	.await;

	let paying_user = auth_session.user_id();

	let receiving_user = if let Some(gift_for) = query.gift_for {
		let receiving_user = global
			.user_loader
			.load_fast(&global, gift_for)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "user not found"))?;

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
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.is_some();
		if is_subscribed {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "user is already subscribed"));
		}

		params.mode = Some(stripe::CheckoutSessionMode::Payment);
		params.payment_intent_data = Some(stripe::CreateCheckoutSessionPaymentIntentData {
			description: Some("Gift subscription payment".to_string()),
			..Default::default()
		});

		params.invoice_creation = Some(stripe::CreateCheckoutSessionInvoiceCreation {
			enabled: true,
			invoice_data: Some(stripe::CreateCheckoutSessionInvoiceCreationInvoiceData {
				metadata: Some(
					InvoiceMetadata::Gift {
						customer_id: paying_user,
						user_id: receiving_user.id,
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
		let is_subscribed = SubscriptionPeriod::collection(&global.db)
			.find_one(filter::filter! {
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
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to load subscription periods");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.is_some();

		if is_subscribed {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "user is already subscribed"));
		}

		params.mode = Some(stripe::CheckoutSessionMode::Subscription);
		params.subscription_data = Some(stripe::CreateCheckoutSessionSubscriptionData {
			metadata: Some(
				SubscriptionMetadata {
					user_id: auth_session.user_id(),
					customer_id: None,
				}
				.to_stripe(),
			),
			..Default::default()
		});

		params.metadata = Some(CheckoutSessionMetadata::Subscription.to_stripe());

		auth_session.user_id()
	};

	// We don't need the safe client here because this won't be retried
	let session_url = stripe::CheckoutSession::create(global.stripe_client.client().await.deref(), params)
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to create checkout session");
			ApiError::INTERNAL_SERVER_ERROR
		})?
		.url
		.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

	Ok(Json(SubscribeResponse {
		url: session_url,
		user_id: receiving_user,
	}))
}
