use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use shared::config::{IncomingRequestConfig, NatsConfig, PodConfig, RateLimit, S3BucketConfig, TlsConfig};

#[derive(Debug, Serialize, Deserialize, smart_default::SmartDefault)]
#[serde(default)]
pub struct Config {
	/// CDN configuration
	pub cdn: Cdn,
	/// NATS configuration
	pub nats: NatsConfig,
	/// Pod configuration
	pub pod: PodConfig,
	/// Log level
	#[default(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))]
	pub level: String,
	/// Metrics bind address
	#[default(None)]
	pub metrics_bind_address: Option<SocketAddr>,
}

#[derive(Debug, Serialize, Deserialize, smart_default::SmartDefault)]
#[serde(default)]
pub struct Cdn {
	/// Bind address
	#[default(SocketAddr::from(([0, 0, 0, 0], 8000)))]
	pub bind: SocketAddr,
	/// Bind address for secure connections, only used if tls is provided.
	#[default(SocketAddr::from(([0, 0, 0, 0], 8443)))]
	pub secure_bind: SocketAddr,
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
	/// Bucket origin
	#[default(S3BucketConfig::default())]
	pub bucket: S3BucketConfig,
	/// Cache capacity in bytes
	#[default(size::Size::from_gigabytes(1))]
	pub cache_capacity: size::Size,
	/// Max concurrent requests to the origin
	#[default(200)]
	pub max_concurrent_requests: u64,
	/// Origin request timeout in seconds
	#[default(5)]
	pub origin_request_timeout: u64,
	/// Rate limit configuration
	#[default(RateLimit::default())]
	pub rate_limit: RateLimit,
	/// IP Header config
	pub incoming_request: IncomingRequestConfig,
	/// NATS Purge Stream
	#[default("cdn.purge".to_string())]
	pub purge_stream_subject: String,
	/// NATS Purge Stream
	#[default("CdnPurge".to_string())]
	pub purge_stream_name: String,
}

scuffle_bootstrap::cli_config!(Config);
