use std::sync::Arc;

use fred::prelude::KeysInterface;

use crate::{
	global::Global,
	http::error::{ApiError, ApiErrorCode},
};

#[derive(serde::Deserialize)]
struct PaypalTokenResponse {
	pub access_token: String,
	pub token_type: String,
	pub expires_in: u64,
}

pub async fn api_key(global: &Arc<Global>) -> Result<String, ApiError> {
	match global.redis.get("paypal_api_key").await.map_err(|err| {
		tracing::error!(error = %err, "failed to get paypal api key");
		ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to get paypal api key")
	})? {
		Some(key) => Ok(key),
		None => {
			let token: PaypalTokenResponse = global
				.http_client
				.post("https://api-m.paypal.com/v1/oauth2/token")
				.basic_auth(&global.config.paypal.client_id, Some(&global.config.paypal.client_secret))
				.form(&[("grant_type", "client_credentials")])
				.send()
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to get new paypal api key");
					ApiError::internal_server_error(ApiErrorCode::PaypalError, "failed to get new paypal api key")
				})?
				.json()
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to decode paypal response");
					ApiError::internal_server_error(ApiErrorCode::PaypalError, "failed to decode paypal response")
				})?;

			if token.token_type != "Bearer" {
				tracing::error!("paypal token type is not Bearer");
				return Err(ApiError::internal_server_error(
					ApiErrorCode::PaypalError,
					"paypal token type is not Bearer",
				));
			}

			// Grace period of 60 seconds
			let ex = token.expires_in.saturating_sub(60) as i64;

			global
				.redis
				.set(
					"paypal_api_key",
					&token.access_token,
					Some(fred::types::Expiration::EX(ex)),
					None,
					false,
				)
				.await
				.map_err(|err| {
					tracing::error!(error = %err, "failed to set paypal api key");
					ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to set paypal api key")
				})?;

			Ok(token.access_token)
		}
	}
}
