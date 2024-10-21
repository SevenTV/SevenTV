use scuffle_foundations::settings::auto_settings;

#[auto_settings]
#[serde(default)]
pub struct NatsConfig {
	/// The URI to use for connecting to Nats
	#[settings(default = vec!["nats://localhost:4222".to_string()])]
	pub servers: Vec<String>,

	/// The username to use for authentication (user-pass auth)
	pub username: Option<String>,

	/// The password to use for authentication (user-pass auth)
	pub password: Option<String>,

	/// The token to use for authentication (token auth)
	pub token: Option<String>,

	/// The TLS configuration (can be used for mTLS)
	pub tls: Option<TlsConfig>,
}

#[auto_settings]
pub struct RedisConfig {
	/// Redis URI
	#[settings(default = vec!["127.0.0.1:6379".to_string()])]
	pub servers: Vec<String>,
	/// Redis username
	#[settings(default = None)]
	pub username: Option<String>,
	/// Redis password
	#[settings(default = None)]
	pub password: Option<String>,
	/// Redis database
	#[settings(default = 0)]
	pub database: u8,
	/// Redis max connections
	#[settings(default = 10)]
	pub max_connections: usize,
	/// Redis TLS configuration
	#[settings(default = None)]
	pub tls: Option<TlsConfig>,
	/// Redis Sentinel configuration
	#[settings(default = None)]
	pub sentinel_service_name: Option<String>,
	#[settings(default = 10)]
	/// The number of redis connections to pool
	pub pool_size: usize,
}

#[auto_settings]
#[serde(default)]
pub struct TlsConfig {
	/// The path to the TLS certificate
	pub cert: String,

	/// The path to the TLS private key
	pub key: String,

	/// The path to the TLS CA certificate
	pub ca_cert: Option<String>,

	/// The alpn protocols to use.
	pub alpn_protocols: Vec<String>,
}

#[auto_settings]
#[serde(default)]
pub struct PodConfig {
	/// Pod name
	pub name: String,
}

#[auto_settings]
#[serde(default)]
pub struct DatabaseConfig {
	/// The URI to use for connecting to the database
	#[settings(default = "mongodb://localhost:27017".into())]
	pub uri: String,
}

#[auto_settings]
#[serde(default)]
pub struct ImageProcessorConfig {
	/// Image Processor address
	pub address: Vec<String>,
	/// Resolve Interval
	#[settings(default = std::time::Duration::from_secs(10))]
	#[serde(with = "humantime_serde")]
	pub resolve_interval: std::time::Duration,
	/// Event Queue Name
	#[settings(default = "nats".into())]
	pub event_queue_name: String,
	/// Event Queue Topic Prefix
	#[settings(default = "image_processor".into())]
	pub event_queue_topic_prefix: String,
	/// Input Drive Name
	#[settings(default = "s3".into())]
	pub input_drive_name: String,
	/// Output Drive Name
	#[settings(default = "s3".into())]
	pub output_drive_name: String,
}

#[auto_settings]
#[serde(default)]
pub struct TypesenseConfig {
	/// The URI to use for connecting to Typesense
	#[settings(default = "http://localhost:8108".into())]
	pub uri: String,

	/// The API key to use for authentication
	pub api_key: Option<String>,
}

#[auto_settings]
#[serde(default)]
pub struct EventStreamConfig {
	/// Replica count for the stream (in nats terms)
	#[settings(default = 1)]
	pub replica_count: i32,

	/// The number of pending acks to buffer
	#[settings(default = 1000)]
	pub ack_capacity: usize,

	/// The prefix to use for the streams created by this application, will be
	/// prepended with a hyphen (if not already present and the stream name is
	/// not empty)
	#[settings(default = "seventv".into())]
	pub stream_prefix: String,
}

#[auto_settings]
#[serde(default)]
pub struct S3BucketConfig {
	/// The name of the S3 bucket
	#[settings(default = String::from("7tv-public"))]
	pub name: String,
	/// The region the S3 bucket is in
	#[settings(default = String::from("us-east-1"))]
	pub region: String,
	/// The custom endpoint for the S3 bucket
	#[settings(default = Some("http://localhost:9000".to_string()))]
	pub endpoint: Option<String>,
	/// The credentials for the S3 bucket
	#[settings(default = S3CredentialsConfig::default())]
	pub credentials: S3CredentialsConfig,
}

#[auto_settings]
#[serde(default)]
pub struct S3CredentialsConfig {
	/// The access key for the S3 bucket
	pub access_key: Option<String>,
	/// The secret key for the S3 bucket
	pub secret_key: Option<String>,
}

impl S3CredentialsConfig {
	pub fn to_credentials(&self) -> Option<aws_sdk_s3::config::Credentials> {
		Some(aws_sdk_s3::config::Credentials::new(
			self.access_key.clone()?,
			self.secret_key.clone()?,
			None,
			None,
			"seventv",
		))
	}
}

#[auto_settings]
#[serde(default)]
pub struct ClickhouseConfig {
	/// Clickhouse URI
	#[settings(default = "http://localhost:8123".into())]
	pub uri: String,

	/// Clickhouse username
	#[settings(default = "default".into())]
	pub username: String,

	/// Clickhouse password
	#[settings(default = "default".into())]
	pub password: String,

	/// Clickhouse database
	#[settings(default = "7tv".into())]
	pub database: String,
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

pub fn default_ipv6_buckets() -> Vec<RateLimitPrefixBucket> {
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

pub fn default_ipv4_buckets() -> Vec<RateLimitPrefixBucket> {
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

pub fn default_range_buckets() -> Vec<RateLimitRangeBucket> {
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

#[auto_settings]
#[serde(default)]
pub struct IncomingRequestConfig {
	/// The IP header to use for incoming requests
	pub ip_header: Option<String>,
	/// A set of trusted proxies that we should use for incoming requests
	pub trusted_proxies: Vec<ipnet::IpNet>,
	/// IP Ranges that are trusted and can ignore the proxy header (if not
	/// provided)
	pub trusted_ranges: Vec<ipnet::IpNet>,
}
