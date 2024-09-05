use scuffle_foundations::bootstrap::{Bootstrap, RuntimeSettings};
use scuffle_foundations::settings::auto_settings;
use scuffle_foundations::telemetry::settings::TelemetrySettings;
use shared::config::{ClickhouseConfig, DatabaseConfig, NatsConfig, TypesenseConfig};

#[auto_settings]
#[serde(default)]
pub struct Config {
	/// Database configuration
	pub database: DatabaseConfig,

	/// NATs configuration
	pub nats: NatsConfig,

	/// Typesense configuration
	pub typesense: TypesenseConfig,

	/// Telemetry configuration
	pub telemetry: TelemetrySettings,

	/// Triggers configuration
	pub triggers: TriggersConfig,

	/// Clickhouse configuration
	pub clickhouse: ClickhouseConfig,
}

#[auto_settings]
#[serde(default)]
pub struct TriggersConfig {
	/// Publish topic
	#[settings(default = "seventv".into())]
	pub nats_prefix: String,

	/// The database name to use for seventv
	#[settings(default = "7tv".into())]
	pub seventv_database: String,

	/// Concurrency limit
	#[settings(default = 10000)]
	pub typesense_concurrency: usize,
}

impl Bootstrap for Config {
	type Settings = Self;

	fn telemetry_config(&self) -> Option<TelemetrySettings> {
		Some(self.telemetry.clone())
	}

	fn runtime_mode(&self) -> RuntimeSettings {
		RuntimeSettings::Steal {
			threads: 0,
			name: "mongo-change-stream".into(),
		}
	}
}
