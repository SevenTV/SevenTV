use std::net::SocketAddr;
use std::time::Duration;

use serde::Deserialize;

use crate::logging;

#[derive(Debug, Deserialize, config::Config, Default)]
#[serde(default)]
pub struct Config<T: config::Config> {
	/// Pod configuration
	pub pod: Pod,
	/// Config file
	pub config_file: Option<String>,
	/// Logging configuration
	pub logging: Logging,
	/// Nats configuration
	pub nats: Nats,
	/// Metrics configuration
	pub metrics: Metrics,
	/// Health configuration
	pub health: Health,
	/// Memory configuration
	pub memory: Memory,

	#[serde(flatten)]
	#[config(flatten)]
	pub extra: T,
}

impl<E: config::Config> std::ops::Deref for Config<E> {
	type Target = E;

	fn deref(&self) -> &Self::Target {
		&self.extra
	}
}

impl<E: config::Config> std::ops::DerefMut for Config<E> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.extra
	}
}

#[derive(Debug, Default, Deserialize, config::Config)]
#[serde(default)]
pub struct Memory {
	/// Memory limit
	pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Logging {
	/// Logging level
	pub level: String,
	/// Logging mode
	pub mode: logging::Mode,
}

impl Default for Logging {
	fn default() -> Self {
		Self {
			level: "info".to_string(),
			mode: logging::Mode::Default,
		}
	}
}

#[derive(Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Nats {
	/// Nats url
	pub url: String,
}

impl Default for Nats {
	fn default() -> Self {
		Self {
			url: "nats://localhost:4222".to_string(),
		}
	}
}

#[derive(Debug, Clone, Default, PartialEq, config::Config, serde::Deserialize)]
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

#[derive(Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Metrics {
	/// Monitoring enabled
	pub enabled: bool,
	/// Http settings
	pub http: Http,
	/// Monitoring labels
	#[config(cli(skip), env(skip))]
	pub labels: Vec<KeyValue>,
}

impl Default for Metrics {
	fn default() -> Self {
		Self {
			enabled: true,
			http: Http::new_with_bind(SocketAddr::new([0, 0, 0, 0].into(), 9090)),
			labels: vec![],
		}
	}
}

#[derive(Debug, Deserialize, config::Config, Default)]
#[serde(default)]
pub struct KeyValue {
	/// Key
	pub key: String,
	/// Value
	pub value: String,
}

#[derive(Debug, Deserialize, Clone, config::Config)]
#[serde(default)]
pub struct Health {
	/// Health enabled
	pub enabled: bool,
	/// http settings
	pub http: Http,
}

impl Default for Health {
	fn default() -> Self {
		Self {
			enabled: true,
			http: Http::new_with_bind(SocketAddr::new([0, 0, 0, 0].into(), 3001)),
		}
	}
}

#[derive(Debug, Deserialize, config::Config, Default)]
#[serde(default)]
pub struct Pod {
	/// Pod name
	pub name: String,
}

#[derive(Debug, Deserialize, Clone, config::Config)]
#[serde(default)]
pub struct Http {
	/// HTTP bind
	pub bind: SocketAddr,
	/// TLS configuration
	pub tls: Option<TlsConfig>,
	/// Max Listen Conn
	pub listen_backlog: u32,
	/// Reuse address
	pub reuse_addr: bool,
	/// Reuse port
	pub reuse_port: bool,
	/// Http1 settings
	pub http1: Http1,
	/// Http2 settings
	pub http2: Http2,
}

impl Http {
	pub fn new_with_bind(bind: SocketAddr) -> Self {
		Self {
			bind,
			tls: None,
			listen_backlog: 128,
			reuse_addr: false,
			reuse_port: false,
			http1: Http1::default(),
			http2: Http2::default(),
		}
	}
}

#[derive(Debug, Deserialize, Clone, config::Config)]
#[serde(default)]
pub struct Http1 {
	/// Enabled
	pub enabled: bool,
	/// Half close
	pub half_close: bool,
	/// Keep alive
	pub keep_alive: bool,
	/// Max buffer size
	pub max_buf_size: usize,
	/// Writev
	pub writev: bool,
	/// Header Read Timeout
	pub header_read_timeout: Option<Duration>,
}

#[derive(Debug, Deserialize, Clone, config::Config)]
#[serde(default)]
pub struct Http2 {
	/// Enabled
	pub enabled: bool,
	/// Max concurrent streams
	pub max_concurrent_streams: u32,
	/// Max frame size
	pub max_frame_size: Option<u32>,
	/// Max header list size
	pub max_header_list_size: u32,
	/// Max send buffer size
	pub max_send_buf_size: usize,
	/// Initial Stream Window Size
	pub initial_stream_window_size: Option<u32>,
	/// Initial Connection Window Size
	pub initial_connection_window_size: Option<u32>,
	/// Adaptive window
	pub adaptive_window: bool,
	/// Keep alive window
	pub keep_alive_interval: Option<Duration>,
	/// Keep alive timeout
	pub keep_alive_timeout: Duration,
}

impl Default for Http1 {
	fn default() -> Self {
		Self {
			enabled: false,
			half_close: true,
			keep_alive: true,
			max_buf_size: 16 * 1024 * 1024,
			writev: true,
			header_read_timeout: None,
		}
	}
}

impl Default for Http2 {
	fn default() -> Self {
		Self {
			enabled: true,
			max_concurrent_streams: 1024,
			max_frame_size: Some(16 * 1024 * 1024),
			max_header_list_size: 16 * 1024 * 1024,
			max_send_buf_size: 16 * 1024 * 1024,
			initial_stream_window_size: None,
			initial_connection_window_size: None,
			adaptive_window: true,
			keep_alive_interval: None,
			keep_alive_timeout: Duration::from_secs(20),
		}
	}
}

impl Default for Http {
	fn default() -> Self {
		Self {
			bind: SocketAddr::new([0, 0, 0, 0].into(), 0),
			tls: None,
			listen_backlog: 128,
			reuse_addr: false,
			reuse_port: false,
			http1: Http1::default(),
			http2: Http2::default(),
		}
	}
}

#[derive(Debug, Deserialize, Clone, config::Config, Default)]
#[serde(default)]
pub struct HttpCors {
	/// Allow headers
	pub allow_headers: Vec<String>,
	/// Allow methods
	pub allow_methods: Vec<String>,
	/// Allow origin
	pub allow_origin: Vec<String>,
	/// Expose headers
	pub expose_headers: Vec<String>,
	/// Max age seconds
	pub max_age_seconds: Option<u64>,
	/// Timing allow origin
	pub timing_allow_origin: Vec<String>,
}

pub fn parse<E: config::Config + serde::de::DeserializeOwned + Default>(
	enable_cli: bool,
	config_file: Option<String>,
) -> config::Result<Config<E>> {
	let mut builder = config::ConfigBuilder::new();

	if enable_cli {
		builder.add_source_with_priority(config::sources::CliSource::new()?, 3);
	}

	builder.add_source_with_priority(config::sources::EnvSource::with_prefix("SEVENTV")?, 2);

	let key = builder.parse_key::<Option<String>>("config_file")?;

	let key_provided = key.is_some();

	let mut config_path = None;

	if let Some(path) = key.or(config_file) {
		match config::sources::FileSource::with_path(path) {
			Ok(source) => {
				config_path = Some(source.location().to_string());
				builder.add_source_with_priority(source, 1);
			}
			Err(err) => {
				if key_provided || !err.is_io() {
					return Err(err);
				}

				tracing::debug!("failed to load config file: {}", err);
			}
		}
	}

	let mut config: Config<E> = builder.build()?;

	config.config_file = config_path.map(|p| std::fs::canonicalize(p).unwrap().display().to_string());

	Ok(config)
}
