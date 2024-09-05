use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;

use super::{create_checkout_session_params, find_or_create_customer};
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::extract::Path;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::rest::users::TargetUser;

#[derive(Debug, serde::Deserialize)]
pub struct PaymentMethodQuery {
	/// always true
	#[serde(rename = "next")]
	_next: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct PaymentMethodResponse {
	/// Url that the website will open in a new tab
	url: String,
}

pub async fn payment_method(
	State(global): State<Arc<Global>>,
	Path(target): Path<TargetUser>,
	Query(_query): Query<PaymentMethodQuery>,
	auth_session: Option<AuthSession>,
) -> Result<Json<PaymentMethodResponse>, ApiError> {
	let auth_session = auth_session.ok_or(ApiError::UNAUTHORIZED)?;

	let user = match target {
		TargetUser::Me => auth_session.user_id(),
		TargetUser::Other(id) => id,
	};

	if user != auth_session.user_id() {
		// TODO: allow with certain permissions
		return Err(ApiError::FORBIDDEN);
	}

	let customer_id = match auth_session.user(&global).await?.stripe_customer_id.clone() {
		Some(id) => id,
		None => find_or_create_customer(&global, auth_session.user_id(), None).await?,
	};

	let mut params = create_checkout_session_params(&global, customer_id, None).await;

	// TODO: make it depend on the user's country
	params.currency = Some(stripe::Currency::EUR);

	params.mode = Some(stripe::CheckoutSessionMode::Setup);

	let success_url = format!("{}/subscribe", global.config.api.website_origin);
	params.success_url = Some(&success_url);

	let cancel_url = format!("{}/subscribe", global.config.api.website_origin);
	params.cancel_url = Some(&cancel_url);

	let url = stripe::CheckoutSession::create(&global.stripe_client, params)
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to create checkout session");
			ApiError::INTERNAL_SERVER_ERROR
		})?
		.url
		.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

	Ok(Json(PaymentMethodResponse { url }))
}
