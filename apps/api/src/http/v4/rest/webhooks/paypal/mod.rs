use std::sync::Arc;

use axum::{
	extract::State,
	http::{HeaderMap, StatusCode},
};
use base64::{prelude::BASE64_STANDARD, Engine};
use rsa::{pkcs1::DecodeRsaPublicKey, traits::SignatureScheme, Pkcs1v15Sign};
use sha2::Digest;
use tokio::sync::OnceCell;

use crate::{global::Global, http::error::ApiError};

mod dispute;
mod sale;
mod subscription;

static PAYPAL_KEY_CACHE: OnceCell<rsa::RsaPublicKey> = OnceCell::const_new();

async fn paypal_key(cert_url: &str) -> Result<&rsa::RsaPublicKey, ApiError> {
	PAYPAL_KEY_CACHE
		.get_or_try_init(|| async {
			let cert_pem = reqwest::get(cert_url)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to download cert");
					ApiError::INTERNAL_SERVER_ERROR
				})?
				.text()
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to read cert");
					ApiError::INTERNAL_SERVER_ERROR
				})?;
			let cert = x509_certificate::X509Certificate::from_pem(&cert_pem).map_err(|e| {
				tracing::error!(error = %e, "failed to parse cert");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

			let public_key = rsa::RsaPublicKey::from_pkcs1_der(&cert.public_key_data()).map_err(|e| {
				tracing::error!(error = %e, "failed to parse public key");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

			Ok(public_key)
		})
		.await
}

/// https://developer.paypal.com/api/rest/webhooks/rest
/// Needlessly complicated because PayPal has a weird way of signing their webhooks
pub async fn handle(
	State(global): State<Arc<Global>>,
	headers: HeaderMap,
	payload: bytes::Bytes,
) -> Result<StatusCode, ApiError> {
	let cert_url = headers
		.get("paypal-cert-url")
		.and_then(|v| v.to_str().ok())
		.ok_or(ApiError::BAD_REQUEST)?;

	let public_key = paypal_key(cert_url).await?;

	let signature = headers
		.get("paypal-transmission-sig")
		.and_then(|v| v.to_str().ok())
		.and_then(|v| BASE64_STANDARD.decode(v).ok())
		.ok_or(ApiError::BAD_REQUEST)?;

	let transmision_id = headers
		.get("paypal-transmission-id")
		.and_then(|v| v.to_str().ok())
		.ok_or(ApiError::BAD_REQUEST)?;

	let webhook_id = &global.config.api.paypal.webhook_id;

	let timestamp = headers
		.get("paypal-transmission-time")
		.and_then(|v| v.to_str().ok())
		.ok_or(ApiError::BAD_REQUEST)?;

	let crc = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
	let mut crc = crc.digest();
	crc.update(&payload);
	let crc = crc.finalize();

	let hash = sha2::Sha256::digest(format!("{transmision_id}|{timestamp}|{webhook_id}|{crc}").as_bytes());

	let scheme = Pkcs1v15Sign::new::<sha2::Sha256>();
	scheme.verify(&public_key, &hash, &signature).map_err(|e| {
		tracing::error!(error = %e, "failed to verify signature");
		ApiError::BAD_REQUEST
	})?;

	tracing::info!("verified signature");

	Ok(StatusCode::OK)
}
