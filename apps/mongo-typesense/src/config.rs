use shared::config::{ClickhouseConfig, DatabaseConfig, NatsConfig, TypesenseConfig};

#[derive(Debug, Clone, serde::Deserialize, smart_default::SmartDefault)]
#[serde(default)]
pub struct Config {
	/// Database configuration
	pub database: DatabaseConfig,

	/// NATs configuration
	pub nats: NatsConfig,

	/// Typesense configuration
	pub typesense: TypesenseConfig,

	/// Triggers configuration
	pub triggers: TriggersConfig,

	/// Clickhouse configuration
	pub clickhouse: ClickhouseConfig,
}

#[derive(Debug, Clone, serde::Deserialize, smart_default::SmartDefault)]
#[serde(default)]
pub struct TriggersConfig {
	/// Publish topic
	#[default("seventv".into())]
	pub nats_prefix: String,

	/// The database name to use for seventv
	#[default("7tv".into())]
	pub seventv_database: String,

	/// Concurrency limit
	#[default(10000)]
	pub typesense_concurrency: usize,
}

scuffle_bootstrap::cli_config!(Config);
