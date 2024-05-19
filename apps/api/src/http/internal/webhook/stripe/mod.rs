use std::sync::Arc;

use axum::extract::{FromRequest, Request, State};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{async_trait, Router};
use hyper::StatusCode;
use stripe::{EventObject, EventType};

use crate::global::Global;
use crate::http::error::ApiError;

mod charge;
mod dispute;
mod invoice;
mod subscription;
mod subscription_schedule;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/", post(handler))
}

struct StripeEvent(stripe::Event);

#[async_trait]
impl FromRequest<Arc<Global>> for StripeEvent
where
	String: FromRequest<()>,
{
	type Rejection = Response;

	async fn from_request(req: Request, global: &Arc<Global>) -> Result<Self, Self::Rejection> {
		let signature = if let Some(sig) = req.headers().get("stripe-signature") {
			sig.to_owned()
		} else {
			return Err(StatusCode::BAD_REQUEST.into_response());
		};

		let payload = String::from_request(req, &()).await.map_err(IntoResponse::into_response)?;

		Ok(Self(
			stripe::Webhook::construct_event(
				&payload,
				signature.to_str().unwrap(),
				&global.config().api.stripe.webhook_secret,
			)
			.map_err(|_| StatusCode::BAD_REQUEST.into_response())?,
		))
	}
}

async fn handler(State(global): State<Arc<Global>>, StripeEvent(event): StripeEvent) -> Result<impl IntoResponse, ApiError> {
	match (event.type_, event.data.object) {
		(EventType::InvoiceCreated, EventObject::Invoice(invoice)) => {
			Ok(invoice::created(global, invoice).await?.into_response())
		}
		(EventType::InvoiceUpdated, EventObject::Invoice(invoice)) => {
			Ok(invoice::updated(global, invoice).await?.into_response())
		}
		(EventType::InvoiceDeleted, EventObject::Invoice(invoice)) => {
			Ok(invoice::deleted(global, invoice).await?.into_response())
		}
		(EventType::InvoicePaid, EventObject::Invoice(invoice)) => Ok(invoice::paid(global, invoice).await?.into_response()),
		(EventType::InvoicePaymentFailed, EventObject::Invoice(invoice)) => {
			Ok(invoice::payment_failed(global, invoice).await?.into_response())
		}
		(EventType::InvoiceVoided, EventObject::Invoice(invoice)) => {
			Ok(invoice::voided(global, invoice).await?.into_response())
		}
		(EventType::InvoiceMarkedUncollectible, EventObject::Invoice(invoice)) => {
			Ok(invoice::marked_uncollectible(global, invoice).await?.into_response())
		}
		(EventType::CustomerSubscriptionCreated, EventObject::Subscription(subscription)) => {
			Ok(subscription::created(global, subscription).await?.into_response())
		}
		(EventType::CustomerSubscriptionUpdated, EventObject::Subscription(subscription)) => {
			Ok(subscription::updated(global, subscription).await?.into_response())
		}
		(EventType::CustomerSubscriptionDeleted, EventObject::Subscription(subscription)) => {
			Ok(subscription::deleted(global, subscription).await?.into_response())
		}
		(EventType::SubscriptionScheduleCreated, EventObject::SubscriptionSchedule(subscription_schedule)) => {
			Ok(subscription_schedule::created(global, subscription_schedule)
				.await?
				.into_response())
		}
		(EventType::SubscriptionScheduleUpdated, EventObject::SubscriptionSchedule(subscription_schedule)) => {
			Ok(subscription_schedule::updated(global, subscription_schedule)
				.await?
				.into_response())
		}
		(EventType::ChargeRefunded, EventObject::Charge(charge)) => {
			Ok(charge::refund(global, charge).await?.into_response())
		}
		(EventType::ChargeDisputeCreated, EventObject::Dispute(dispute)) => {
			Ok(dispute::created(global, dispute).await?.into_response())
		}
		(EventType::ChargeDisputeUpdated, EventObject::Dispute(dispute)) => {
			Ok(dispute::updated(global, dispute).await?.into_response())
		}
		(EventType::ChargeDisputeClosed, EventObject::Dispute(dispute)) => {
			Ok(dispute::closed(global, dispute).await?.into_response())
		}
		_ => {
			tracing::warn!("unhandled event type: {:?}", event.type_);
			Err(ApiError::BAD_REQUEST)
		}
	}
}
