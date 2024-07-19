use scuffle_foundations::bootstrap::{Bootstrap, RuntimeSettings};
use scuffle_foundations::settings::auto_settings;
use scuffle_foundations::telemetry::settings::TelemetrySettings;
use shared::config::{DatabaseConfig, NatsConfig};

#[auto_settings]
#[serde(default)]
pub struct Config {
	/// Database configuration
	pub database: DatabaseConfig,

	/// NATs configuration
	pub nats: NatsConfig,

	/// Telemetry configuration
	pub telemetry: TelemetrySettings,

	/// Publish topic
	#[settings(default = "seventv".into())]
	pub nats_prefix: String,

	/// Nats back pressure
	#[settings(default = 1000)]
	pub back_pressure: usize,
}

impl Bootstrap for Config {
	type Settings = Self;

	fn telemetry_config(&self) -> Option<TelemetrySettings> {
		Some(self.telemetry.clone())
	}

	fn runtime_mode(&self) -> RuntimeSettings {
		RuntimeSettings::NoSteal {
			threads: 1,
			name: "mongo-change-stream".into(),
		}
	}
}
