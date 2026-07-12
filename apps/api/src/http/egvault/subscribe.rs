use std::ops::Deref;
use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde::Deserialize;
use shared::database::product::{SubscriptionProduct, SubscriptionProductKind, SubscriptionProductVariant};
use shared::database::queries::filter;
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, UserPermission};
use shared::database::user::UserId;
use shared::database::MongoCollection;

use super::metadata::{CheckoutSessionMetadata, InvoiceMetadata, StripeMetadata, SubscriptionMetadata, MAX_GIFT_RECIPIENTS};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::extract::Query;
use crate::http::middleware::session::Session;
use crate::ratelimit::RateLimitRequest;
use crate::stripe_common::{create_checkout_session_params, find_or_create_customer, resolve_months, CheckoutProduct, Prefill};

fn default_months() -> u32 {
	1
}

fn deserialize_gift_for<'de, D>(deserializer: D) -> Result<Vec<(UserId, u32)>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	let raw = Option::<String>::deserialize(deserializer)?;
	match raw {
		None => Ok(vec![]),
		Some(raw) if raw.is_empty() => Ok(vec![]),
		Some(raw) => raw
			.split(',')
			.map(|entry| {
				let entry = entry.trim();
				match entry.split_once(':') {
					Some((id, months)) => {
						let id = id.parse::<UserId>().map_err(serde::de::Error::custom)?;
						let months = months.parse::<u32>().map_err(serde::de::Error::custom)?;
						Ok((id, months))
					}
					None => {
						let id = entry.parse::<UserId>().map_err(serde::de::Error::custom)?;
						Ok((id, default_months()))
					}
				}
			})
			.collect(),
	}
}

#[derive(Debug, serde::Deserialize)]
pub struct SubscribeQuery {
	renew_interval: SubscriptionRenewInterval,
	/// only "stripe" allowed
	payment_method: String,
	/// always true
	#[serde(rename = "next")]
	_next: bool,
	#[serde(default, deserialize_with = "deserialize_gift_for")]
	gift_for: Vec<(UserId, u32)>,
	#[serde(default = "default_months")]
	months: u32,
}

#[derive(Debug, serde::Deserialize)]
pub struct SubscribeBody {
	prefill: Prefill,
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
	/// The user id(s) of the user(s) that receive(s) the subscription
	user_ids: Vec<UserId>,
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

	if query.gift_for.len() > MAX_GIFT_RECIPIENTS {
		return Err(ApiError::bad_request(
			ApiErrorCode::BadRequest,
			format!("cannot gift to more than {MAX_GIFT_RECIPIENTS} recipients at once"),
		));
	}

	let mut dedup_check = query.gift_for.iter().map(|(id, _)| *id).collect::<Vec<_>>();
	dedup_check.sort();
	dedup_check.dedup();
	if dedup_check.len() != query.gift_for.len() {
		return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "duplicate gift_for recipient"));
	}

	let kind = SubscriptionProductKind::from(query.renew_interval);
	let is_gift = !query.gift_for.is_empty();

	let recipients: Vec<(UserId, u32)> = if is_gift {
		query
			.gift_for
			.iter()
			.map(|(id, months)| Ok((*id, resolve_months(*months, &kind)?)))
			.collect::<Result<Vec<_>, ApiError>>()?
	} else {
		vec![(authed_user.id, resolve_months(query.months, &kind)?)]
	};

	let is_one_time_purchase = is_gift || recipients.iter().any(|(_, months)| *months > 1);

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

		let success_url = global
			.config
			.api
			.old_website_origin
			.join("/subscribe/complete?with_provider=stripe")
			.unwrap()
			.to_string();
		let cancel_url = global
			.config
			.api
			.old_website_origin
			.join("/subscribe/cancel?with_provider=stripe")
			.unwrap()
			.to_string();

		let quantities = recipients.iter().map(|(_, months)| *months as u64).collect::<Vec<_>>();

		let mut params = create_checkout_session_params(
			&global,
			session.ip(),
			customer_id,
			if is_one_time_purchase {
				CheckoutProduct::Gift(product.provider_id, quantities)
			} else {
				CheckoutProduct::Price(variant.id.0.clone(), 1)
			},
			product.default_currency,
			&variant.currency_prices,
			&success_url,
			&cancel_url,
		)
		.await;

		if is_one_time_purchase {
			let mut loaded_recipients = Vec::with_capacity(recipients.len());
			for (recipient_id, months) in &recipients {
				let recipient = global
					.user_loader
					.load_fast(&global, *recipient_id)
					.await
					.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
					.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;
				loaded_recipients.push((recipient, *months));
			}

			let description = if is_gift {
				let names = loaded_recipients
					.iter()
					.map(|(r, months)| {
						let name = r
							.connections
							.first()
							.map(|c| format!("{} ({}:{})", c.platform_display_name, c.platform, c.platform_id))
							.unwrap_or_else(|| format!("7TV:{}", r.id));
						format!("{name} ({months} month(s))")
					})
					.collect::<Vec<_>>()
					.join(", ");

				format!("Gift 7TV Subscriber to {names}")
			} else {
				let months = recipients.first().map(|(_, months)| *months).unwrap_or(1);
				format!("{months} month(s) of 7TV Subscriber")
			};

			params.mode = Some(stripe::CheckoutSessionMode::Payment);
			params.payment_intent_data = Some(stripe::CreateCheckoutSessionPaymentIntentData {
				description: Some(description),
				..Default::default()
			});

			params.invoice_creation = Some(stripe::CreateCheckoutSessionInvoiceCreation {
				enabled: true,
				invoice_data: Some(stripe::CreateCheckoutSessionInvoiceCreationInvoiceData {
					metadata: Some(
						InvoiceMetadata::Gift {
							customer_id: authed_user.id,
							recipients: recipients.clone(),
							product_id: variant.id.clone(),
							subscription_product_id: Some(product.id),
						}
						.to_stripe(),
					),
					..Default::default()
				}),
			});

			params.metadata = Some(CheckoutSessionMetadata::Gift.to_stripe());
		} else {
			let is_subscribed = global
				.active_subscription_period_by_user_id_loader
				.load(authed_user.id)
				.await
				.map_err(|()| {
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription period")
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
			user_ids: recipients.into_iter().map(|(id, _)| id).collect(),
		}))
	})
	.await
}
