use std::ops::Deref;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, UserPermission};

use super::metadata::{CheckoutSessionMetadata, StripeMetadata};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::extract::Path;
use crate::http::middleware::session::Session;
use crate::http::v3::rest::users::TargetUser;
use crate::ratelimit::RateLimitRequest;
use crate::stripe_common::find_or_create_customer;

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
	Extension(session): Extension<Session>,
) -> Result<impl IntoResponse, ApiError> {
	let auth_user = session.user()?;

	let target_id = match target {
		TargetUser::Me => auth_user.id,
		TargetUser::Other(id) => id,
	};

	let target = global
		.user_loader
		.load(&global, target_id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load target user"))?
		.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "target user not found"))?;

	if !target.has(UserPermission::Billing) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"this user isn't allowed to use billing features",
		));
	}

	if target_id != auth_user.id && !auth_user.has(UserPermission::ManageBilling) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"you are not allowed to manage billing",
		));
	}

	let target_user = global
		.user_loader
		.load_fast(&global, target_id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
		.ok_or_else(|| ApiError::not_found(ApiErrorCode::BadRequest, "user not found"))?;

	let req = RateLimitRequest::new(RateLimitResource::EgVaultPaymentMethod, &session);

	req.http(&global, async {
		let customer_id = match target_user.stripe_customer_id.clone() {
			Some(id) => id,
			// We don't need the safe client here because this won't be retried
			None => find_or_create_customer(&global, global.stripe_client.client().await, target_user.id, None).await?,
		};

		let callback = global.config.api.website_origin.join("subscribe").unwrap();

		let mut currency = stripe::Currency::EUR;

		if let Some(country_code) = global.geoip().and_then(|g| g.lookup(session.ip())).and_then(|c| c.iso_code) {
			if let Ok(Some(global)) = global.global_config_loader.load(()).await {
				if let Some(currency_override) = global.country_currency_overrides.get(country_code) {
					currency = *currency_override;
				}
			}
		}

		let params = stripe::CreateCheckoutSession {
			line_items: None,
			mode: Some(stripe::CheckoutSessionMode::Setup),
			customer_update: Some(stripe::CreateCheckoutSessionCustomerUpdate {
				address: Some(stripe::CreateCheckoutSessionCustomerUpdateAddress::Auto),
				..Default::default()
			}),
			currency: Some(currency),
			customer: Some(customer_id.into()),
			// expire the session 4 hours from now so we can restore unused redeem codes in the checkout.session.expired
			// handler
			expires_at: Some((chrono::Utc::now() + chrono::Duration::hours(4)).timestamp()),
			success_url: Some(callback.as_str()),
			cancel_url: Some(callback.as_str()),
			metadata: Some(CheckoutSessionMetadata::Setup.to_stripe()),
			..Default::default()
		};

		// We don't need the safe client here because this won't be retried
		let url = stripe::CheckoutSession::create(global.stripe_client.client().await.deref(), params)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to create checkout session");
				ApiError::internal_server_error(ApiErrorCode::StripeError, "failed to create checkout session")
			})?
			.url
			.ok_or_else(|| ApiError::internal_server_error(ApiErrorCode::StripeError, "checkout session url is missing"))?;

		Ok::<_, ApiError>(Json(PaymentMethodResponse { url }))
	})
	.await
}
