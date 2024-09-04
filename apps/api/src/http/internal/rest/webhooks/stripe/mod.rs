use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use mongodb::options::UpdateOptions;
use shared::database::queries::{filter, update};
use shared::database::webhook_event::WebhookEvent;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::sub_refresh_job;
use crate::transactions::{with_transaction, TransactionError};

mod charge;
mod checkout_session;
mod customer;
mod invoice;
mod subscription;

pub async fn handle(State(global): State<Arc<Global>>, headers: HeaderMap, payload: String) -> Result<StatusCode, ApiError> {
	let sig = headers
		.get("stripe-signature")
		.and_then(|v| v.to_str().ok())
		.ok_or(ApiError::BAD_REQUEST)?;

	let event = stripe::Webhook::construct_event(&payload, sig, &global.config.api.stripe.webhook_secret).map_err(|e| {
		tracing::error!(error = %e, "failed to parse webhook");
		ApiError::BAD_REQUEST
	})?;

	// TODO: verify request is coming from stripe ip
	// https://docs.stripe.com/ips#webhook-notifications

	let res = with_transaction(&global, |mut tx| {
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
							id: event.id.to_string(),
							created_at: chrono::DateTime::from_timestamp(event.created, 0)
								.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?,
						},
					},
					UpdateOptions::builder().upsert(true).build(),
				)
				.await?;

			if res.matched_count > 0 {
				// already processed
				return Ok(None);
			}

			let prev_attributes = event.data.previous_attributes;

			// https://kappa.lol/BB4ME
			match (event.type_, event.data.object) {
				(stripe::EventType::CustomerCreated, stripe::EventObject::Customer(cus)) => {
					customer::created(&global, tx, cus).await?;
				}
				(stripe::EventType::CheckoutSessionCompleted, stripe::EventObject::CheckoutSession(s)) => {
					return checkout_session::completed(&global, tx, s).await;
				}
				(stripe::EventType::CheckoutSessionExpired, stripe::EventObject::CheckoutSession(s)) => {
					checkout_session::expired(&global, tx, s).await?;
				}
				(stripe::EventType::InvoiceCreated, stripe::EventObject::Invoice(iv)) => {
					invoice::created(&global, tx, iv).await?;
				}
				(stripe::EventType::InvoiceUpdated, stripe::EventObject::Invoice(iv))
				| (stripe::EventType::InvoiceFinalized, stripe::EventObject::Invoice(iv))
				| (stripe::EventType::InvoicePaymentSucceeded, stripe::EventObject::Invoice(iv)) => {
					invoice::updated(
						&global,
						&mut tx,
						&iv,
						prev_attributes.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?,
					)
					.await?;
				}
				(stripe::EventType::InvoicePaid, stripe::EventObject::Invoice(iv)) => {
					return invoice::paid(&global, tx, iv).await;
				}
				(stripe::EventType::InvoiceDeleted, stripe::EventObject::Invoice(iv)) => {
					invoice::deleted(&global, tx, iv).await?;
				}
				(stripe::EventType::InvoicePaymentFailed, stripe::EventObject::Invoice(iv)) => {
					invoice::payment_failed(&global, tx, iv).await?;
				}
				(stripe::EventType::CustomerSubscriptionCreated, stripe::EventObject::Subscription(sub)) => {
					return subscription::created(&global, tx, sub).await;
				}
				(stripe::EventType::CustomerSubscriptionUpdated, stripe::EventObject::Subscription(sub)) => {
					return subscription::updated(
						&global,
						tx,
						chrono::DateTime::from_timestamp(event.created, 0)
							.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?,
						sub,
						prev_attributes.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?,
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
					charge::dispute_updated(&global, tx, dis).await?;
				}
				_ => return Err(TransactionError::custom(ApiError::BAD_REQUEST)),
			}

			Ok(None)
		}
	})
	.await;

	match res {
		Ok(Some(sub_id)) => {
			sub_refresh_job::refresh(&global, &sub_id).await?;
			Ok(StatusCode::OK)
		}
		Ok(None) => Ok(StatusCode::OK),
		Err(TransactionError::Custom(e)) => Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			Err(ApiError::INTERNAL_SERVER_ERROR)
		}
	}
}
