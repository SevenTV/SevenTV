use scuffle_foundations::bootstrap::{Bootstrap, RuntimeSettings};
use scuffle_foundations::settings::auto_settings;
use scuffle_foundations::telemetry::settings::TelemetrySettings;
use shared::config::{DatabaseConfig, NatsConfig, TypesenseConfig};

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
}

#[auto_settings]
#[serde(default)]
pub struct TriggersConfig {
	/// Publish topic
	#[settings(default = "mongo-change-stream".into())]
	pub topic: String,

	/// The database name to use for seventv
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
