use scuffle_foundations::bootstrap::RuntimeSettings;
use scuffle_foundations::settings::{auto_settings, Settings};
use scuffle_foundations::telementry::settings::TelementrySettings;

#[auto_settings]
pub struct Config<T: Settings + Default> {
	/// Pod configuration
	pub pod: Pod,
	/// Nats configuration
	pub nats: Nats,
	/// Telementry configuration
	pub telementry: TelementrySettings,
	/// Runtime configuration
	pub runtime: RuntimeSettings,
	#[serde(flatten)]
	pub extra: T,
}

impl<E: Settings + Default> std::ops::Deref for Config<E> {
	type Target = E;

	fn deref(&self) -> &Self::Target {
		&self.extra
	}
}

impl<E: Settings + Default> std::ops::DerefMut for Config<E> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.extra
	}
}

#[auto_settings]
#[serde(default)]
pub struct Nats {
	/// The URI to use for connecting to Nats
	#[settings(default = vec!["nats://localhost:4222".to_string()])]
	pub servers: Vec<String>,

	/// The username to use for authentication (user-pass auth)
	pub username: Option<String>,

	/// The password to use for authentication (user-pass auth)
	pub password: Option<String>,

	/// The token to use for authentication (token auth)
	pub token: Option<String>,

	/// The TLS configuration (can be used for mTLS)
	pub tls: Option<TlsConfig>,
}

#[auto_settings]
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

#[auto_settings]
#[serde(default)]
pub struct Pod {
	/// Pod name
	pub name: String,
}

#[auto_settings]
#[serde(default)]
pub struct DatabaseConfig {
	/// The URI to use for connecting to the database
	#[settings(default = "mongodb://localhost:27017".into())]
	pub uri: String,
}
