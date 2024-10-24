use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use mongodb::options::UpdateOptions;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::traits::SignatureScheme;
use rsa::Pkcs1v15Sign;
use sha2::Digest;
use shared::database::queries::{filter, update};
use shared::database::webhook_event::WebhookEvent;
use tokio::sync::Mutex;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::sub_refresh_job;
use crate::transactions::{transaction_with_mutex, TransactionError};

mod dispute;
mod sale;
mod subscription;
pub mod types;

async fn paypal_key(cert_url: &str) -> Result<rsa::RsaPublicKey, ApiError> {
	static PAYPAL_KEY_CACHE: OnceLock<Mutex<HashMap<String, rsa::RsaPublicKey>>> = OnceLock::new();

	let cache = PAYPAL_KEY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

	let mut lock = cache.lock().await;

	if let Some(key) = lock.get(cert_url) {
		return Ok(key.clone());
	}

	if !cert_url.starts_with("https://api.paypal.com/") {
		tracing::warn!(url = %cert_url, "invalid cert url");
		return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "invalid cert url"));
	}

	let cert_pem = reqwest::get(cert_url)
		.await
		.map_err(|e| {
			tracing::error!(url = %cert_url, error = %e, "failed to download cert");
			ApiError::internal_server_error(ApiErrorCode::PaypalError, "failed to download cert")
		})?
		.text()
		.await
		.map_err(|e| {
			tracing::error!(url = %cert_url, error = %e, "failed to read cert");
			ApiError::internal_server_error(ApiErrorCode::PaypalError, "failed to read cert")
		})?;
	let cert = x509_certificate::X509Certificate::from_pem(&cert_pem).map_err(|e| {
		tracing::error!(url = %cert_url, error = %e, "failed to parse cert");
		ApiError::internal_server_error(ApiErrorCode::PaypalError, "failed to parse cert")
	})?;

	if !cert.time_constraints_valid(None) {
		tracing::warn!(url = %cert_url, not_after = %cert.validity_not_after(), not_before = %cert.validity_not_before(), "cert is expired or not yet valid");
		return Err(ApiError::internal_server_error(
			ApiErrorCode::PaypalError,
			"cert is expired or not yet valid",
		));
	}

	// We don't have to verify the certificate because we know it's coming from a
	// paypal domain.

	let public_key = rsa::RsaPublicKey::from_pkcs1_der(&cert.public_key_data()).map_err(|e| {
		tracing::error!(url = %cert_url, error = %e, "failed to parse public key");
		ApiError::internal_server_error(ApiErrorCode::PaypalError, "failed to parse public key")
	})?;

	lock.insert(cert_url.to_string(), public_key.clone());

	Ok(public_key)
}

#[derive(Debug, Clone)]
enum PaypalMutexKey {
	Sale(String),
	Dispute(String),
	Subscription(String),
}

impl PaypalMutexKey {
	fn from_paypal(resource: &types::Resource) -> Self {
		match resource {
			types::Resource::Sale(sale) => Self::Sale(sale.id.clone()),
			types::Resource::Refund(refund) => Self::Sale(refund.sale_id.clone()),
			types::Resource::Dispute(dispute) => Self::Dispute(dispute.dispute_id.clone()),
			types::Resource::Subscription(subscription) => Self::Subscription(subscription.id.clone()),
		}
	}
}

impl std::fmt::Display for PaypalMutexKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		const PREFIX: &str = "mutex:internal:webhook:paypal";

		match self {
			Self::Sale(id) => write!(f, "{PREFIX}:{}", id),
			Self::Dispute(id) => write!(f, "{PREFIX}:{}", id),
			Self::Subscription(id) => write!(f, "{PREFIX}:{}", id),
		}
	}
}

/// https://developer.paypal.com/api/rest/webhooks/rest
/// Needlessly complicated because PayPal has a weird way of signing their
/// webhooks
#[tracing::instrument(skip_all, name = "webhook::paypal", fields(event_id, event_type))]
pub async fn handle(
	State(global): State<Arc<Global>>,
	headers: HeaderMap,
	payload: bytes::Bytes,
) -> Result<StatusCode, ApiError> {
	let cert_url = headers
		.get("paypal-cert-url")
		.and_then(|v| v.to_str().ok())
		.ok_or_else(|| ApiError::bad_request(ApiErrorCode::BadRequest, "missing or invalid paypal-cert-url header"))?;

	let public_key = paypal_key(cert_url).await?;

	let signature = headers
		.get("paypal-transmission-sig")
		.and_then(|v| v.to_str().ok())
		.and_then(|v| BASE64_STANDARD.decode(v).ok())
		.ok_or_else(|| {
			ApiError::bad_request(ApiErrorCode::BadRequest, "missing or invalid paypal-transmission-sig header")
		})?;

	let transmision_id = headers
		.get("paypal-transmission-id")
		.and_then(|v| v.to_str().ok())
		.ok_or_else(|| {
			ApiError::bad_request(ApiErrorCode::BadRequest, "missing or invalid paypal-transmission-id header")
		})?;

	let webhook_id = &global.config.paypal.webhook_id;

	let timestamp = headers
		.get("paypal-transmission-time")
		.and_then(|v| v.to_str().ok())
		.ok_or_else(|| {
			ApiError::bad_request(ApiErrorCode::BadRequest, "missing or invalid paypal-transmission-time header")
		})?;

	let crc = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
	let mut crc = crc.digest();
	crc.update(&payload);
	let crc = crc.finalize();

	let hash = sha2::Sha256::digest(format!("{transmision_id}|{timestamp}|{webhook_id}|{crc}").as_bytes());

	let scheme = Pkcs1v15Sign::new::<sha2::Sha256>();
	scheme.verify(&public_key, &hash, &signature).map_err(|e| {
		tracing::error!(error = %e, "failed to verify signature");
		ApiError::bad_request(ApiErrorCode::PaypalError, "failed to verify signature")
	})?;

	let event: types::Event = serde_json::from_slice(&payload).map_err(|e| {
		tracing::error!(error = %e, "failed to deserialize payload");
		ApiError::bad_request(ApiErrorCode::PaypalError, "failed to deserialize payload")
	})?;

	tracing::Span::current().record("event_id", &event.id);
	tracing::Span::current().record("event_type", event.event_type.as_str());

	let stripe_client = global.stripe_client.safe(&event.id).await;

	let global = &global;

	let mutex_key = PaypalMutexKey::from_paypal(&event.resource);

	let res = transaction_with_mutex(global, Some(mutex_key.into()), |mut tx| {
		async move {
			let res = tx
				.update_one(
					filter::filter! {
						WebhookEvent {
							#[query(rename = "_id")]
							id: &event.id,
						}
					},
					update::update! {
						#[query(set_on_insert)]
						WebhookEvent {
							#[query(rename = "_id")]
							id: &event.id,
							expires_at: event.create_time + chrono::Duration::weeks(1),
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
				tracing::info!("paypal event already processed");
				return Ok(None);
			}

			tracing::info!("processing paypal event");

			match (event.event_type, event.resource) {
				(types::EventType::PaymentSaleCompleted, types::Resource::Sale(sale)) => {
					return sale::completed(global, stripe_client, tx, sale).await;
				}
				(types::EventType::PaymentSaleReversed, types::Resource::Refund(refund))
				| (types::EventType::PaymentSaleRefunded, types::Resource::Refund(refund)) => sale::refunded(global, tx, refund).await?,
				(types::EventType::CustomerDisputeCreated, types::Resource::Dispute(dispute))
				| (types::EventType::CustomerDisputeUpdated, types::Resource::Dispute(dispute))
				| (types::EventType::CustomerDisputeResolved, types::Resource::Dispute(dispute)) => {
					dispute::updated(global, tx, dispute).await?
				}
				(types::EventType::BillingSubscriptionCancelled, types::Resource::Subscription(subscription))
				| (types::EventType::BillingSubscriptionSuspended, types::Resource::Subscription(subscription)) => {
					return subscription::cancelled(global, tx, *subscription).await;
				}
				_ => {
					tracing::warn!(event_type = ?event.event_type, "unsupported event type");
					return Err(TransactionError::Custom(ApiError::bad_request(
						ApiErrorCode::BadRequest,
						"invalid event type",
					)));
				}
			}

			Ok(None)
		}
	})
	.await;

	match res {
		Ok(Some(sub_id)) => {
			sub_refresh_job::refresh(global, sub_id).await?;
			Ok(StatusCode::OK)
		}
		Ok(None) => Ok(StatusCode::OK),
		Err(TransactionError::Custom(e)) => Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			Err(ApiError::internal_server_error(
				ApiErrorCode::TransactionError,
				"transaction failed",
			))
		}
	}
}
