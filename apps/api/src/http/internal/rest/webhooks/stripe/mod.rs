use std::net::IpAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Extension;
use mongodb::options::UpdateOptions;
use shared::database::queries::{filter, update};
use shared::database::webhook_event::WebhookEvent;
use tokio::sync::OnceCell;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::sub_refresh_job;
use crate::transactions::{transaction_with_mutex, TransactionError};

mod charge;
mod checkout_session;
mod customer;
mod invoice;
mod price;
mod subscription;

#[derive(Debug, Clone)]
enum StripeMutexKey {
	Invoice(stripe::InvoiceId),
	Subscription(stripe::SubscriptionId),
	Charge(stripe::ChargeId),
	Price(stripe::PriceId),
	Dispute(stripe::DisputeId),
	Customer(stripe::CustomerId),
	CheckoutSession(stripe::CheckoutSessionId),
}

impl StripeMutexKey {
	fn from_stripe(value: &stripe::EventObject) -> Option<Self> {
		match value {
			stripe::EventObject::Invoice(iv) => Some(Self::Invoice(iv.id.clone())),
			stripe::EventObject::Subscription(sub) => Some(Self::Subscription(sub.id.clone())),
			stripe::EventObject::Charge(ch) => Some(Self::Charge(ch.id.clone())),
			stripe::EventObject::Price(p) => Some(Self::Price(p.id.clone())),
			stripe::EventObject::Dispute(d) => Some(Self::Dispute(d.id.clone())),
			stripe::EventObject::Customer(c) => Some(Self::Customer(c.id.clone())),
			stripe::EventObject::CheckoutSession(s) => Some(Self::CheckoutSession(s.id.clone())),
			_ => None,
		}
	}
}

impl std::fmt::Display for StripeMutexKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		const PREFIX: &str = "mutex:internal:webook:stripe";
		match self {
			Self::Invoice(id) => write!(f, "{PREFIX}:invoice:{}", id),
			Self::Subscription(id) => write!(f, "{PREFIX}:subscription:{}", id),
			Self::Charge(id) => write!(f, "{PREFIX}:charge:{}", id),
			Self::Price(id) => write!(f, "{PREFIX}:price:{}", id),
			Self::Dispute(id) => write!(f, "{PREFIX}:dispute:{}", id),
			Self::Customer(id) => write!(f, "{PREFIX}:customer:{}", id),
			Self::CheckoutSession(id) => write!(f, "{PREFIX}:checkout_session:{}", id),
		}
	}
}

/// https://docs.stripe.com/ips#webhook-notifications
async fn verify_stripe_ip(global: &Arc<Global>, ip: &IpAddr) -> bool {
	#[derive(serde::Deserialize)]
	struct Response {
		#[serde(rename = "WEBHOOKS")]
		webhooks: Vec<IpAddr>,
	}

	static STRIPE_IPS: OnceCell<Response> = OnceCell::const_new();

	match STRIPE_IPS
		.get_or_try_init(|| async {
			global
				.http_client
				.get("https://stripe.com/files/ips/ips_webhooks.json")
				.send()
				.await?
				.json()
				.await
		})
		.await
	{
		Ok(res) => res.webhooks.contains(ip),
		Err(e) => {
			tracing::error!(err = %e, "failed to fetch stripe ip list");
			false
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum StripeRequest {
	Price,
	CheckoutSession(checkout_session::StripeRequest),
	Invoice(invoice::StripeRequest),
	Charge,
}

impl std::fmt::Display for StripeRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Price => write!(f, "price"),
			Self::CheckoutSession(r) => write!(f, "checkout_session:{}", r),
			Self::Invoice(r) => write!(f, "invoice:{}", r),
			Self::Charge => write!(f, "charge"),
		}
	}
}

#[tracing::instrument(skip_all, fields(event_id, event_type), name = "webhook:stripe")]
pub async fn handle(
	State(global): State<Arc<Global>>,
	Extension(session): Extension<Session>,
	headers: HeaderMap,
	payload: String,
) -> Result<StatusCode, ApiError> {
	let sig = headers
		.get("stripe-signature")
		.and_then(|v| v.to_str().ok())
		.ok_or_else(|| ApiError::bad_request(ApiErrorCode::BadRequest, "missing stripe-signature header"))?;

	let event = stripe::Webhook::construct_event(&payload, sig, &global.config.stripe.webhook_secret).map_err(|e| {
		tracing::error!(error = %e, "failed to parse webhook");
		ApiError::bad_request(ApiErrorCode::StripeError, "failed to parse webhook")
	})?;

	tracing::Span::current().record("event_id", event.id.to_string());
	tracing::Span::current().record("event_type", event.type_.to_string());

	if !verify_stripe_ip(&global, &session.ip()).await {
		return Err(ApiError::bad_request(
			ApiErrorCode::BadRequest,
			format!("invalid ip: {}", session.ip()),
		));
	}

	let stripe_client = global.stripe_client.safe(&event.id).await;

	let mutex_key = StripeMutexKey::from_stripe(&event.data.object);

	let res = transaction_with_mutex(&global, mutex_key.map(Into::into), |mut tx| {
		let global = Arc::clone(&global);

		async move {
			let res = tx
				.update_one(
					filter::filter! {
						WebhookEvent {
							#[query(rename = "_id")]
							id: event.id.to_string(),
						}
					},
					update::update! {
						#[query(set_on_insert)]
						WebhookEvent {
							#[query(rename = "_id")]
							id: event.id.to_string(),
							expires_at: chrono::DateTime::from_timestamp(event.created, 0)
								.ok_or_else(|| TransactionError::Custom(ApiError::bad_request(ApiErrorCode::StripeError, "webhook event created_at is missing")))? + chrono::Duration::weeks(1),
						},
						#[query(inc)]
						WebhookEvent {
							received_count: 1,
						},
					},
					UpdateOptions::builder().upsert(true).build(),
				)
				.await?;

			if res.upserted_id.is_none() {
				// already processed
				tracing::info!("stripe event already processed");
				return Ok(None);
			}

			tracing::info!("processing stripe event");

			let prev_attributes = event.data.previous_attributes;

			// https://kappa.lol/2NwAU
			match (event.type_, event.data.object) {
				(stripe::EventType::PriceUpdated, stripe::EventObject::Price(price)) => {
					price::updated(&global, stripe_client, tx, price).await?;
				}
				(stripe::EventType::PriceDeleted, stripe::EventObject::Price(price)) => {
					price::deleted(&global, tx, price).await?;
				}
				(stripe::EventType::CustomerCreated, stripe::EventObject::Customer(cus)) => {
					customer::created(&global, tx, cus).await?;
				}
				(stripe::EventType::CheckoutSessionCompleted, stripe::EventObject::CheckoutSession(s)) => {
					return checkout_session::completed(&global, stripe_client, tx, s).await;
				}
				(stripe::EventType::CheckoutSessionExpired, stripe::EventObject::CheckoutSession(s)) => {
					checkout_session::expired(&global, tx, s).await?;
				}
				(stripe::EventType::InvoiceCreated, stripe::EventObject::Invoice(iv)) => {
					invoice::created(&global, stripe_client, tx, iv).await?;
				}
				(stripe::EventType::InvoiceUpdated, stripe::EventObject::Invoice(iv))
				| (stripe::EventType::InvoiceFinalized, stripe::EventObject::Invoice(iv))
				| (stripe::EventType::InvoicePaymentSucceeded, stripe::EventObject::Invoice(iv)) => {
					invoice::updated(&global, &mut tx, &iv).await?;
				}
				(stripe::EventType::InvoicePaid, stripe::EventObject::Invoice(iv)) => {
					return invoice::paid(&global, stripe_client, tx, event.id, iv).await;
				}
				(stripe::EventType::InvoiceDeleted, stripe::EventObject::Invoice(iv)) => {
					invoice::deleted(&global, tx, iv).await?;
				}
				(stripe::EventType::InvoicePaymentFailed, stripe::EventObject::Invoice(iv)) => {
					invoice::payment_failed(&global, tx, iv).await?;
				}
				(stripe::EventType::CustomerSubscriptionCreated, stripe::EventObject::Subscription(sub)) => {
					return subscription::created(&global, tx, sub, event.id).await;
				}
				(stripe::EventType::CustomerSubscriptionUpdated, stripe::EventObject::Subscription(sub)) => {
					return subscription::updated(
						&global,
						tx,
						event.id,
						chrono::DateTime::from_timestamp(event.created, 0)
							.ok_or_else(|| TransactionError::Custom(ApiError::bad_request(ApiErrorCode::StripeError, "webhook event created_at is missing")))?,
						sub,
						prev_attributes.unwrap_or_default(),
					)
					.await;
				}
				(stripe::EventType::CustomerSubscriptionDeleted, stripe::EventObject::Subscription(sub)) => {
					return subscription::deleted(&global, tx, sub).await;
				}
				(stripe::EventType::ChargeRefunded, stripe::EventObject::Charge(ch)) => {
					charge::refunded(&global, tx, ch).await?;
				}
				(stripe::EventType::ChargeDisputeCreated, stripe::EventObject::Dispute(dis))
				| (stripe::EventType::ChargeDisputeClosed, stripe::EventObject::Dispute(dis))
				| (stripe::EventType::ChargeDisputeUpdated, stripe::EventObject::Dispute(dis)) => {
					charge::dispute_updated(&global, stripe_client, tx, dis).await?;
				}
				_ => return Err(TransactionError::Custom(ApiError::bad_request(ApiErrorCode::StripeError, "invalid event type"))),
			}

			Ok(None)
		}
	})
	.await;

	match res {
		Ok(Some(sub_id)) => {
			sub_refresh_job::refresh(&global, sub_id).await?;
			Ok(StatusCode::OK)
		}
		Ok(None) => Ok(StatusCode::OK),
		Err(TransactionError::Custom(e)) => Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			Err(ApiError::bad_request(ApiErrorCode::TransactionError, "transaction failed"))
		}
	}
}
