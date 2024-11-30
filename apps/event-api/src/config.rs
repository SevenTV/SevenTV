use std::net::SocketAddr;
use std::time::Duration;

use shared::config::{IncomingRequestConfig, NatsConfig, PodConfig, RateLimit, TlsConfig};

#[derive(Clone, Debug, smart_default::SmartDefault, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Config {
	/// Pod configuration
	pub pod: PodConfig,
	/// Nats configuration
	pub nats: NatsConfig,
	/// EventApi configuration
	pub event_api: EventApi,
	/// Metrics bind address
	#[default(None)]
	pub metrics_bind: Option<SocketAddr>,
	/// Log level
	#[default(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))]
	pub level: String,
}

#[derive(Clone, Debug, smart_default::SmartDefault, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct EventApi {
	/// Bind address
	#[default(SocketAddr::from(([0, 0, 0, 0], 8000)))]
	pub bind: SocketAddr,
	/// The number of workers handling requests
	#[default(1)]
	pub workers: usize,
	/// With Http3
	pub http3: bool,
	/// The server name to use for the CDN
	#[default("SevenTV".into())]
	pub server_name: String,
	/// Allow insecure connections to the CDN (only used if tls is provided)
	#[default(false)]
	pub allow_insecure: bool,
	/// A TLS configuration for the CDN
	pub tls: Option<TlsConfig>,
	/// API heartbeat interval
	#[default(Duration::from_secs(45))]
	pub heartbeat_interval: Duration,
	/// API subscription limit
	#[default(Some(500))]
	pub subscription_limit: Option<usize>,
	/// API connection limit
	pub connection_limit: Option<usize>,
	/// API connection time limit
	#[default(Duration::from_secs(60 * 60))]
	pub ttl: Duration,
	/// API bridge url
	#[default("http://localhost:9700".to_string())]
	pub bridge_url: String,
	/// Cdn Origin
	#[default("https://cdn.7tv.app".parse().unwrap())]
	pub cdn_origin: url::Url,
	/// Rate limit configuration
	#[default(RateLimit::default())]
	pub rate_limit: RateLimit,
	/// IP Header config
	pub incoming_request: IncomingRequestConfig,
}

scuffle_bootstrap::cli_config!(Config);
