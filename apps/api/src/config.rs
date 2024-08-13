use std::net::SocketAddr;
use std::path::PathBuf;

use scuffle_foundations::bootstrap::{Bootstrap, RuntimeSettings};
use scuffle_foundations::settings::auto_settings;
use scuffle_foundations::telemetry::settings::TelemetrySettings;
use shared::config::{DatabaseConfig, ImageProcessorConfig, NatsConfig, TypesenseConfig};

#[auto_settings]
#[serde(default)]
pub struct Api {
	/// http options
	#[settings(default = SocketAddr::from(([0, 0, 0, 0], 8080)))]
	pub bind: SocketAddr,
	/// worker count
	#[settings(default = 1)]
	pub workers: usize,
	/// website origin
	#[settings(default = "https://7tv.app".into())]
	pub website_origin: String,
	/// cdn base url
	#[settings(default = "//cdn.7tv.app".into())]
	pub cdn_origin: String,
	/// public domain
	#[settings(default = "7tv.io".into())]
	pub domain: String,
	/// base url
	#[settings(default = "https://7tv.io".into())]
	pub api_origin: String,
	/// connection config
	pub connections: ConnectionsConfig,
	/// jwt config
	pub jwt: JwtConfig,
	/// image processor config
	pub image_processor: ImageProcessorConfig,
	/// Event API nats prefix
	#[settings(default = "api.events".into())]
	pub nats_event_subject: String,
}

#[auto_settings]
#[serde(default)]
pub struct JwtConfig {
	/// JWT secret
	#[settings(default = "seventv-api".into())]
	pub secret: String,

	/// JWT issuer
	#[settings(default = "seventv-api".into())]
	pub issuer: String,
}

#[auto_settings]
#[serde(default)]
pub struct ConnectionsConfig {
	/// Twitch connection
	pub twitch: ConnectionConfig,
	/// Discord connection
	pub discord: ConnectionConfig,
	/// Google connection
	pub google: ConnectionConfig,
}

#[auto_settings]
#[serde(default)]
pub struct ConnectionConfig {
	/// Client ID
	#[settings(default = "client_id".into())]
	pub client_id: String,
	/// Client Secret
	#[settings(default = "client_secret".into())]
	pub client_secret: String,
}

#[auto_settings]
#[serde(default)]
pub struct ChangeStreamConfig {
	/// Change Stream Prefix
	#[settings(default = "seventv".into())]
	pub prefix: String,

	/// The number of pending acks to buffer
	#[settings(default = 1000)]
	pub back_pressure_limit: usize,
}

#[auto_settings]
#[serde(default)]
pub struct Config {
	/// Export GQL schema
	#[settings(default = None)]
	pub export_schema_path: Option<PathBuf>,

	/// API configuration
	pub api: Api,

	/// Database configuration
	pub database: DatabaseConfig,

	/// Database configuration
	pub clickhouse: DatabaseConfig,

	/// NATs configuration
	pub nats: NatsConfig,

	/// Typesense configuration
	pub typesense: TypesenseConfig,

	/// Telemetry configuration
	pub telemetry: TelemetrySettings,

	/// Runtime configuration
	pub runtime: RuntimeSettings,
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
