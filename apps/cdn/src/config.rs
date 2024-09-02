use std::net::SocketAddr;

use scuffle_foundations::bootstrap::{Bootstrap, RuntimeSettings};
use scuffle_foundations::settings::auto_settings;
use scuffle_foundations::telemetry::settings::TelemetrySettings;
use shared::config::{S3BucketConfig, TlsConfig};

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
		let mut settings = self.telemetry.clone();
		settings
			.metrics
			.labels
			.entry("server_name".to_string())
			.or_insert(self.cdn.server_name.clone());
		if let Ok(host_info) = sys_metrics::host::get_host_info() {
			settings
				.metrics
				.labels
				.entry("system".to_string())
				.or_insert(host_info.system);
			settings
				.metrics
				.labels
				.entry("kernel_version".to_string())
				.or_insert(host_info.kernel_version);
			settings
				.metrics
				.labels
				.entry("hostname".to_string())
				.or_insert(host_info.hostname);
		}
		Some(settings)
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
}

#[auto_settings]
#[serde(default)]
pub struct RateLimit {
	#[settings(default = default_ipv6_buckets())]
	pub ipv6_buckets: Vec<RateLimitPrefixBucket>,
	#[settings(default = default_ipv4_buckets())]
	pub ipv4_buckets: Vec<RateLimitPrefixBucket>,
	#[settings(default = default_range_buckets())]
	pub range_buckets: Vec<RateLimitRangeBucket>,
}

fn default_ipv6_buckets() -> Vec<RateLimitPrefixBucket> {
	vec![
		RateLimitPrefixBucket {
			prefix_length: 128,
			concurrent_connections: 50,
		},
		RateLimitPrefixBucket {
			prefix_length: 64,
			concurrent_connections: 100,
		},
		RateLimitPrefixBucket {
			prefix_length: 48,
			concurrent_connections: 1000,
		},
		RateLimitPrefixBucket {
			prefix_length: 32,
			concurrent_connections: 10000,
		},
	]
}

fn default_ipv4_buckets() -> Vec<RateLimitPrefixBucket> {
	vec![
		RateLimitPrefixBucket {
			prefix_length: 32,
			concurrent_connections: 125,
		},
		RateLimitPrefixBucket {
			prefix_length: 24,
			concurrent_connections: 250,
		},
		RateLimitPrefixBucket {
			prefix_length: 16,
			concurrent_connections: 10000,
		},
	]
}

fn default_range_buckets() -> Vec<RateLimitRangeBucket> {
	vec![
		// private ipv4
		RateLimitRangeBucket {
			range: "10.0.0.0/8".parse().unwrap(),
			concurrent_connections: None,
		},
		// private ipv4
		RateLimitRangeBucket {
			range: "172.16.0.0/12".parse().unwrap(),
			concurrent_connections: None,
		},
		// private ipv4
		RateLimitRangeBucket {
			range: "192.168.0.0/16".parse().unwrap(),
			concurrent_connections: None,
		},
		// loopback ipv4
		RateLimitRangeBucket {
			range: "127.0.0.0/8".parse().unwrap(),
			concurrent_connections: None,
		},
		// private ipv6
		RateLimitRangeBucket {
			range: "fc00::/7".parse().unwrap(),
			concurrent_connections: None,
		},
		// link local ipv6
		RateLimitRangeBucket {
			range: "fe80::/10".parse().unwrap(),
			concurrent_connections: None,
		},
		// ipv6 loopback
		RateLimitRangeBucket {
			range: "::1/128".parse().unwrap(),
			concurrent_connections: None,
		},
	]
}

#[auto_settings]
#[derive(Copy)]
pub struct RateLimitRangeBucket {
	pub range: ipnet::IpNet,
	pub concurrent_connections: Option<u64>,
}

#[auto_settings]
#[derive(Copy)]
pub struct RateLimitPrefixBucket {
	pub prefix_length: u8,
	pub concurrent_connections: u64,
}
