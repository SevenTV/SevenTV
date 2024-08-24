use std::sync::Arc;

use axum::{
	extract::State,
	http::{HeaderMap, StatusCode},
};

use crate::{global::Global, http::error::ApiError};

mod charge;
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

	// TODO: implement deduplication based on the event id
	// TODO: verify request is coming from stripe ip
	// https://docs.stripe.com/ips#webhook-notifications

	match (event.type_, event.data.object) {
		(stripe::EventType::InvoiceCreated, stripe::EventObject::Invoice(iv)) => invoice::created(&global, iv).await,
		(stripe::EventType::InvoiceUpdated, stripe::EventObject::Invoice(iv)) => invoice::updated(&global, iv).await,
		(stripe::EventType::InvoiceDeleted, stripe::EventObject::Invoice(iv)) => invoice::deleted(&global, iv).await,
		(stripe::EventType::InvoicePaid, stripe::EventObject::Invoice(iv)) => invoice::paid(&global, iv).await,
		(stripe::EventType::InvoicePaymentFailed, stripe::EventObject::Invoice(iv)) => {
			invoice::payment_failed(&global, iv).await
		}
		(stripe::EventType::InvoiceVoided, stripe::EventObject::Invoice(iv)) => invoice::voided(&global, iv).await,
		(stripe::EventType::InvoiceMarkedUncollectible, stripe::EventObject::Invoice(iv)) => {
			invoice::marked_uncollectible(&global, iv).await
		}
		(stripe::EventType::CustomerSubscriptionCreated, stripe::EventObject::Subscription(sub)) => {
			subscription::created(&global, sub).await
		}
		(stripe::EventType::CustomerSubscriptionDeleted, stripe::EventObject::Subscription(sub)) => {
			subscription::deleted(&global, sub).await
		}
		(stripe::EventType::CustomerSubscriptionUpdated, stripe::EventObject::Subscription(sub)) => {
			subscription::updated(&global, sub).await
		}
		(stripe::EventType::ChargeRefunded, stripe::EventObject::Charge(ch)) => charge::refunded(&global, ch).await,
		(stripe::EventType::ChargeDisputeCreated, stripe::EventObject::Charge(ch)) => {
			charge::dispute_created(&global, ch).await
		}
		(stripe::EventType::ChargeDisputeUpdated, stripe::EventObject::Charge(ch)) => {
			charge::dispute_updated(&global, ch).await
		}
		(stripe::EventType::ChargeDisputeClosed, stripe::EventObject::Charge(ch)) => {
			charge::dispute_closed(&global, ch).await
		}
		_ => Ok(StatusCode::BAD_REQUEST),
	}
}
