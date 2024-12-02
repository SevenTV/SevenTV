use shared::config::{DatabaseConfig, NatsConfig};

#[derive(Debug, smart_default::SmartDefault, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Config {
	/// Database configuration
	pub database: DatabaseConfig,

	/// NATs configuration
	pub nats: NatsConfig,

	/// Publish topic
	#[default("seventv".into())]
	pub nats_prefix: String,

	/// Nats back pressure
	#[default(1000)]
	pub back_pressure: usize,

	/// Log level
	#[default(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))]
	pub level: String,

	/// Metrics bind
	#[default(None)]
	pub metrics_bind_address: Option<std::net::SocketAddr>,
}

scuffle_settings::bootstrap!(Config);
