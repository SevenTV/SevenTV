use std::{collections::HashMap, sync::Arc};

use axum::{
	extract::State,
	http::{HeaderMap, StatusCode},
};
use base64::{prelude::BASE64_STANDARD, Engine};
use rsa::{pkcs1::DecodeRsaPublicKey, traits::SignatureScheme, Pkcs1v15Sign};
use sha2::Digest;
use shared::database::webhook_event::WebhookEvent;
use tokio::sync::{OnceCell, RwLock};

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{with_transaction, TransactionError},
};

mod dispute;
mod sale;
mod subscription;
pub mod types;

async fn paypal_key(cert_url: &str) -> Result<rsa::RsaPublicKey, ApiError> {
	static PAYPAL_KEY_CACHE: OnceCell<RwLock<HashMap<String, rsa::RsaPublicKey>>> = OnceCell::const_new();

	let cache = PAYPAL_KEY_CACHE.get_or_init(|| async { RwLock::new(HashMap::new()) }).await;

	if let Some(key) = cache.read().await.get(cert_url) {
		return Ok(key.clone());
	}

	if !cert_url.starts_with("https://api.paypal.com/") {
		tracing::warn!(url = %cert_url, "invalid cert url");
		return Err(ApiError::BAD_REQUEST);
	}

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

	if cert.time_constraints_valid(None) {
		tracing::warn!("cert is expired or not yet valid");
		return Err(ApiError::BAD_REQUEST);
	}

	// TODO: check if cert has a valid signature

	let public_key = rsa::RsaPublicKey::from_pkcs1_der(&cert.public_key_data()).map_err(|e| {
		tracing::error!(error = %e, "failed to parse public key");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	cache.write().await.insert(cert_url.to_string(), public_key.clone());

	Ok(public_key)
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

	let event: types::Event = serde_json::from_slice(&payload).map_err(|e| {
		tracing::error!(error = %e, "failed to deserialize payload");
		ApiError::BAD_REQUEST
	})?;

	let res = with_transaction(&global, |mut tx| {
		let global = Arc::clone(&global);

		async move {
			tx.insert_one(
				WebhookEvent {
					id: event.id,
					created_at: event.create_time,
				},
				None,
			)
			.await?;

			match (event.event_type, event.ressource) {
				(types::EventType::PaymentSaleCompleted, types::Resource::Sale(sale)) => {
					sale::completed(&global, tx, sale).await
				}
				(types::EventType::PaymentSaleRefunded, types::Resource::Sale(sale)) => {
					sale::refunded(&global, tx, sale).await
				}
				(types::EventType::PaymentSaleReversed, types::Resource::Sale(sale)) => {
					sale::reversed(&global, tx, sale).await
				}
				(types::EventType::CustomerDisputeCreated, types::Resource::Dispute(dispute)) => {
					dispute::created(&global, tx, dispute).await
				}
				(types::EventType::CustomerDisputeUpdated, types::Resource::Dispute(dispute)) => {
					dispute::updated(&global, tx, dispute).await
				}
				(types::EventType::CustomerDisputeResolved, types::Resource::Dispute(dispute)) => {
					dispute::resolved(&global, tx, dispute).await
				}
				(types::EventType::BillingSubscriptionExpired, types::Resource::Subscription(subscription)) => {
					subscription::expired(&global, tx, subscription).await
				}
				(types::EventType::BillingSubscriptionCancelled, types::Resource::Subscription(subscription)) => {
					subscription::cancelled(&global, tx, subscription).await
				}
				(types::EventType::BillingSubscriptionSuspended, types::Resource::Subscription(subscription)) => {
					subscription::suspended(&global, tx, subscription).await
				}
				(types::EventType::BillingSubscriptionPaymentFailed, types::Resource::Subscription(subscription)) => {
					subscription::payment_failed(&global, tx, subscription).await
				}
				_ => Err(TransactionError::custom(ApiError::BAD_REQUEST)),
			}
		}
	})
	.await;

	match res {
		Ok(_) => Ok(StatusCode::OK),
		Err(TransactionError::Custom(e)) => Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			Err(ApiError::INTERNAL_SERVER_ERROR)
		}
	}
}
