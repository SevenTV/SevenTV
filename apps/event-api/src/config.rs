use std::net::SocketAddr;
use std::time::Duration;

use serde::Deserialize;
use shared::config::{Http, HttpCors};

#[derive(Debug, Deserialize, config::Config, Default)]
#[serde(default)]
pub struct Extra {
	/// Api configuration
	pub api: Api,
}

#[derive(Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Api {
	/// Http options
	pub http: Http,
	/// Cors options
	pub cors: Option<HttpCors>,
	/// API heartbeat interval
	pub heartbeat_interval: Duration,
	/// Subscription Cleanup Interval
	pub subscription_cleanup_interval: Duration,
	/// API subscription limit
	pub subscription_limit: Option<usize>,
	/// API connection limit
	pub connection_limit: Option<usize>,
	/// API connection target
	pub connection_target: Option<usize>,
	/// API connection time limit
	pub ttl: Duration,
	/// API bridge url
	pub bridge_url: String,
	/// Nats Event Subject
	pub nats_event_subject: String,
}

impl Default for Api {
	fn default() -> Self {
		Self {
			http: Http::new_with_bind(SocketAddr::from(([0, 0, 0, 0], 3000))),
			cors: None,
			connection_limit: None,
			connection_target: None,
			heartbeat_interval: Duration::from_secs(45),
			subscription_cleanup_interval: Duration::from_secs(60 * 2),
			subscription_limit: Some(500),
			ttl: Duration::from_secs(60 * 60),
			bridge_url: "http://localhost:9700".to_string(),
			nats_event_subject: "api.events".to_string(),
		}
	}
}

pub type Config = shared::config::Config<Extra>;
