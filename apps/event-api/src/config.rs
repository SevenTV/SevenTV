use std::net::SocketAddr;
use std::time::Duration;

use scuffle_foundations::bootstrap::{Bootstrap, RuntimeSettings};
use scuffle_foundations::settings::auto_settings;
use scuffle_foundations::telemetry::settings::TelemetrySettings;
use shared::config::{IncomingRequestConfig, NatsConfig, PodConfig, RateLimit, TlsConfig};

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
	/// EventApi configuration
	pub event_api: EventApi,
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
pub struct EventApi {
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
	/// API heartbeat interval
	#[settings(default = Duration::from_secs(45))]
	pub heartbeat_interval: Duration,
	/// API subscription limit
	#[settings(default = Some(500))]
	pub subscription_limit: Option<usize>,
	/// API connection limit
	pub connection_limit: Option<usize>,
	/// API connection time limit
	#[settings(default = Duration::from_secs(60 * 60))]
	pub ttl: Duration,
	/// API bridge url
	#[settings(default = "http://localhost:9700".to_string())]
	pub bridge_url: String,
	/// Cdn Origin
	#[settings(default = "https://cdn.7tv.app".parse().unwrap())]
	pub cdn_origin: url::Url,
	/// Rate limit configuration
	#[settings(default = RateLimit::default())]
	pub rate_limit: RateLimit,
	/// IP Header config
	pub incoming_request: IncomingRequestConfig,
}
