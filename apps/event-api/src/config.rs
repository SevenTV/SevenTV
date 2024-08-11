use std::net::SocketAddr;
use std::time::Duration;

use scuffle_foundations::bootstrap::{Bootstrap, RuntimeSettings};
use scuffle_foundations::settings::auto_settings;
use scuffle_foundations::telemetry::settings::TelemetrySettings;
use shared::config::{NatsConfig, PodConfig};

#[auto_settings]
#[serde(default)]
pub struct Config {
	/// Pod configuration
	pub pod: PodConfig,
	/// Nats configuration
	pub nats: NatsConfig,
	/// Telemetry configuration
	pub telemetry: TelemetrySettings,
	/// Runtime configuration
	pub runtime: RuntimeSettings,
	/// Api configuration
	pub api: Api,
}

impl Bootstrap for Config {
	type Settings = Self;

	fn telemetry_config(&self) -> Option<TelemetrySettings> {
		Some(self.telemetry.clone())
	}

	fn runtime_mode(&self) -> RuntimeSettings {
		self.runtime.clone()
	}
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
	/// Cdn Origin
	#[settings(default = "https://cdn.7tv.app".to_string())]
	pub cdn_origin: String,
}
