use std::sync::Arc;

use moka::future::Cache;
use scc::TreeIndex;
use scuffle_foundations::telemetry::server::HealthCheck;

use crate::{
	cache::{self, CacheKey, CachedResponse, PathMeta},
	config::Config,
};

pub struct Global {
	pub config: Config,
	pub http_client: reqwest::Client,
	pub cache: Cache<CacheKey, CachedResponse>,
	pub path_meta: TreeIndex<String, Arc<PathMeta>>,
}

impl Global {
	pub async fn new(config: Config) -> Self {
		Self {
			config,
			http_client: reqwest::Client::new(),
			cache: cache::create(),
			path_meta: TreeIndex::new(),
		}
	}
}

impl HealthCheck for Global {
	fn check(&self) -> std::pin::Pin<Box<dyn futures::Future<Output = bool> + Send + '_>> {
		Box::pin(async {
			tracing::info!("running health check");

			true
		})
	}
}
