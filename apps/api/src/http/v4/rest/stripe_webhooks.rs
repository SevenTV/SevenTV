use std::sync::Arc;

use axum::{
	extract::State,
	http::{HeaderMap, StatusCode},
};

use crate::{global::Global, http::error::ApiError};

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
		(stripe::EventType::InvoiceCreated, stripe::EventObject::Invoice(iv)) => handle_invoice_created(&global, iv).await,
		_ => Ok(StatusCode::BAD_REQUEST),
	}
}

async fn handle_invoice_created(global: &Arc<Global>, invoice: stripe::Invoice) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::OK)
}
