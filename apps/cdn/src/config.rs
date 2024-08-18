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
	#[settings(default = S3BucketConfig::default())]
	pub bucket: S3BucketConfig,
	/// Cache capacity
	#[settings(default = 1000)]
	pub cache_capacity: u64,
}

#[auto_settings]
#[serde(default)]
pub struct S3CredentialsConfig {
	/// The access key for the S3 bucket
	pub access_key: Option<String>,

	/// The secret key for the S3 bucket
	pub secret_key: Option<String>,
}

impl S3CredentialsConfig {
	pub fn to_credentials(&self) -> Option<aws_sdk_s3::config::Credentials> {
		Some(aws_sdk_s3::config::Credentials::new(
			self.access_key.clone()?,
			self.secret_key.clone()?,
			None,
			None,
			"seventv-cdn",
		))
	}
}

#[auto_settings]
#[serde(default)]
pub struct S3BucketConfig {
	/// The name of the S3 bucket
	#[settings(default = String::from("7tv-cdn"))]
	pub name: String,
	/// The region the S3 bucket is in
	#[settings(default = String::from("us-east-1"))]
	pub region: String,
	/// The custom endpoint for the S3 bucket
	#[settings(default = Some("http://localhost:9000".to_string()))]
	pub endpoint: Option<String>,
	/// The credentials for the S3 bucket
	#[settings(default = S3CredentialsConfig::default())]
	pub credentials: S3CredentialsConfig,
}
