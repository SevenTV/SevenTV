use std::net::SocketAddr;

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
	/// Monitoring configuration
	pub monitoring: Monitoring,
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
	/// Nats subject
	pub subject: String,
}

impl Default for Nats {
	fn default() -> Self {
		Self {
			url: "nats://localhost:4222".to_string(),
			subject: "events".to_string(),
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
}

#[derive(Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Monitoring {
	/// Monitoring enabled
	pub enabled: bool,
	/// Monitoring bind
	pub bind: SocketAddr,
	/// Monitoring labels
	#[config(cli(skip), env(skip))]
	pub labels: Vec<KeyValue>,
}

impl Default for Monitoring {
	fn default() -> Self {
		Self {
			enabled: true,
			bind: SocketAddr::new([0, 0, 0, 0].into(), 3002),
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

#[derive(Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Health {
	/// Health enabled
	pub enabled: bool,
	/// Health bind
	pub bind: SocketAddr,
}

impl Default for Health {
	fn default() -> Self {
		Self {
			enabled: true,
			bind: SocketAddr::new([0, 0, 0, 0].into(), 3001),
		}
	}
}

#[derive(Debug, Deserialize, config::Config, Default)]
#[serde(default)]
pub struct Pod {
	/// Pod name
	pub name: String,
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
