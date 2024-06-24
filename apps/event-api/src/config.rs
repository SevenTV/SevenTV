use std::{net::SocketAddr, time::Duration};

use scuffle_foundations::settings::auto_settings;

#[auto_settings]
#[serde(default)]
pub struct Extra {
	/// Api configuration
	pub api: Api,
}

#[auto_settings]
#[serde(default)]
pub struct Api {
	/// bind
	#[settings(default = SocketAddr::from(([0, 0, 0, 0], 3000)))]
	pub bind: SocketAddr,
	/// Cors options
	// pub cors: Option<HttpCors>,
	/// API heartbeat interval
	#[settings(default = Duration::from_secs(45))]
	pub heartbeat_interval: Duration,
	/// Subscription Cleanup Interval
	#[settings(default = Duration::from_secs(60 * 2))]
	pub subscription_cleanup_interval: Duration,
	/// API subscription limit
	#[settings(default = Some(500))]
	pub subscription_limit: Option<usize>,
	/// API connection limit
	pub connection_limit: Option<usize>,
	/// API connection target
	pub connection_target: Option<usize>,
	/// API connection time limit
	#[settings(default = Duration::from_secs(60 * 60))]
	pub ttl: Duration,
	/// API bridge url
	#[settings(default = "http://localhost:9700".to_string())]
	pub bridge_url: String,
	/// Nats Event Subject
	#[settings(default = "api.events".to_string())]
	pub nats_event_subject: String,
}

pub type Config = shared::config::Config<Extra>;
