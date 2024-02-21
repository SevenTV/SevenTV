use std::net::SocketAddr;
use std::time::Duration;

use serde::Deserialize;
use shared::config::TlsConfig;

#[derive(Debug, Deserialize, config::Config, Default)]
#[serde(default)]
pub struct Extra {
	/// Api configuration
	pub api: Api,
}

#[derive(Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Api {
	/// API bind
	pub bind: SocketAddr,
	/// Max Listen Conn
	pub listen_backlog: u32,
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
	/// API v3 enabled
	pub v3: bool,
	/// API bridge url
	pub bridge_url: String,
	/// TLS configuration
	pub tls: Option<TlsConfig>,
}

impl Default for Api {
	fn default() -> Self {
		Self {
			bind: SocketAddr::new([0, 0, 0, 0].into(), 3000),
			listen_backlog: 128,
			connection_limit: None,
			connection_target: None,
			heartbeat_interval: Duration::from_secs(45),
			subscription_cleanup_interval: Duration::from_secs(60 * 2),
			subscription_limit: Some(500),
			ttl: Duration::from_secs(60 * 60),
			v3: true,
			bridge_url: "http://localhost:9700".to_string(),
			tls: None,
		}
	}
}

pub type Config = shared::config::Config<Extra>;
