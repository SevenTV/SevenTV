use std::net::SocketAddr;

use scuffle_foundations::bootstrap::{Bootstrap, RuntimeSettings};
use scuffle_foundations::settings::auto_settings;
use scuffle_foundations::telemetry::settings::TelemetrySettings;
use shared::config::S3BucketConfig;

#[auto_settings]
#[serde(default)]
pub struct Config {
	/// Telemetry configuration
	pub telemetry: TelemetrySettings,
	/// Runtime configuration
	pub runtime: RuntimeSettings,
	/// Api configuration
	pub cdn: Cdn,
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
pub struct Cdn {
	/// Bind address
	#[settings(default = SocketAddr::from(([0, 0, 0, 0], 8000)))]
	pub bind: SocketAddr,
	/// Bucket origin
	#[settings(default = S3BucketConfig::default())]
	pub bucket: S3BucketConfig,
	/// Cache capacity in bytes
	#[settings(default = 1024 * 1024 * 1024)]
	pub cache_capacity: u64,
	/// Max concurrent requests to the origin
	#[settings(default = 200)]
	pub max_concurrent_requests: u64,
	/// Origin request timeout in seconds
	#[settings(default = 5)]
	pub origin_request_timeout: u64,
}
