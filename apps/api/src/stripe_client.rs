use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use crate::config::Config;

// stripe::Client is cheap to clone
// https://github.com/arlyon/async-stripe/issues/520

#[derive(Clone)]
pub struct StripeClientManager {
	/// cheap to clone
	client: stripe::Client,
	semaphore: Arc<tokio::sync::Semaphore>,
}

impl StripeClientManager {
	pub fn new(config: &Config) -> Self {
		Self {
			client: stripe::Client::new(&config.api.stripe.api_key).with_app_info(
				env!("CARGO_PKG_NAME").to_string(),
				Some(env!("CARGO_PKG_VERSION").to_string()),
				Some(config.api.api_origin.to_string()),
			),
			semaphore: Arc::new(tokio::sync::Semaphore::new(config.api.stripe.concurrent_requests)),
		}
	}

	/// This function returns a stripe client without idempotency.
	/// Idempotency is handled by the `SafeStripeClient` returned by `safe`.
	///
	/// Use the safe client for all requests that could potentially be retried.
	/// (e.g. in a database transaction)
	pub async fn client(&self) -> StripeClient {
		let permit = Arc::clone(&self.semaphore).acquire_owned().await.expect("semaphore closed");

		StripeClient {
			inner: self.client.clone(),
			_permit: permit,
		}
	}

	/// This function returns a safe stripe client with idempotency.
	/// The safe client should be used for all requests that could potentially
	/// be retried. (e.g. in a database transaction)
	pub async fn safe(&self) -> SafeStripeClient {
		SafeStripeClient {
			idempotency: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
			semaphore: Arc::clone(&self.semaphore),
			client: self.client.clone(),
		}
	}
}

#[derive(Clone)]
pub struct SafeStripeClient {
	idempotency: Arc<tokio::sync::Mutex<HashMap<u32, stripe::RequestStrategy>>>,
	client: stripe::Client,
	semaphore: Arc<tokio::sync::Semaphore>,
}

impl SafeStripeClient {
	/// This function returns a stripe client.
	/// The `key` should be the same for all requests that should be considered
	/// the same by stripe.
	pub async fn client(&self, key: u32) -> StripeClient {
		let permit = Arc::clone(&self.semaphore).acquire_owned().await.expect("semaphore closed");

		let strategy = self
			.idempotency
			.lock()
			.await
			.entry(key)
			.or_insert_with(|| stripe::RequestStrategy::idempotent_with_uuid())
			.clone();

		let inner = self.client.clone().with_strategy(strategy);

		StripeClient { inner, _permit: permit }
	}
}

pub struct StripeClient {
	inner: stripe::Client,
	_permit: tokio::sync::OwnedSemaphorePermit,
}

impl Deref for StripeClient {
	type Target = stripe::Client;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}
