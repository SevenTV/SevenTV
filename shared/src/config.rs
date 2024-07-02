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

	/// The prefix to use for the streams created by this application, will be prepended with a hyphen (if not already present and the stream name is not empty)
	#[settings(default = "seventv".into())]
	pub stream_prefix: String,
}
