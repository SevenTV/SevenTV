use std::net::SocketAddr;

use scuffle_foundations::bootstrap::{Bootstrap, RuntimeSettings};
use scuffle_foundations::settings::auto_settings;
use scuffle_foundations::telemetry::settings::TelemetrySettings;

#[auto_settings]
#[serde(default)]
pub struct Config {
	/// Telemetry configuration
	pub telemetry: TelemetrySettings,
	/// Runtime configuration
	pub runtime: RuntimeSettings,
	/// Api configuration
	pub cdn: Cdn,
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

#[auto_settings]
#[serde(default)]
pub struct Cdn {
	/// Bind address
	#[settings(default = SocketAddr::from(([0, 0, 0, 0], 8000)))]
	pub bind: SocketAddr,
    /// Bucket origin
	pub bucket_url: String,
}
