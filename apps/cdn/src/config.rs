use std::net::SocketAddr;

use scuffle_foundations::bootstrap::{Bootstrap, RuntimeSettings};
use scuffle_foundations::settings::auto_settings;
use scuffle_foundations::telemetry::settings::TelemetrySettings;
use shared::config::{IncomingRequestConfig, RateLimit, S3BucketConfig, TlsConfig};

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
	/// Bind address for secure connections, only used if tls is provided.
	#[settings(default = SocketAddr::from(([0, 0, 0, 0], 8443)))]
	pub secure_bind: SocketAddr,
	/// The number of workers handling requests
	#[settings(default = 1)]
	pub workers: usize,
	/// With Http3
	pub http3: bool,
	/// The server name to use for the CDN
	#[settings(default = "SevenTV".into())]
	pub server_name: String,
	/// Allow insecure connections to the CDN (only used if tls is provided)
	#[settings(default = false)]
	pub allow_insecure: bool,
	/// A TLS configuration for the CDN
	pub tls: Option<TlsConfig>,
	/// Bucket origin
	#[settings(default = S3BucketConfig::default())]
	pub bucket: S3BucketConfig,
	/// Cache capacity in bytes
	#[settings(default = size::Size::from_gigabytes(1))]
	pub cache_capacity: size::Size,
	/// Max concurrent requests to the origin
	#[settings(default = 200)]
	pub max_concurrent_requests: u64,
	/// Origin request timeout in seconds
	#[settings(default = 5)]
	pub origin_request_timeout: u64,
	/// Rate limit configuration
	#[settings(default = RateLimit::default())]
	pub rate_limit: RateLimit,
	/// IP Header config
	pub incoming_request: IncomingRequestConfig,
}
