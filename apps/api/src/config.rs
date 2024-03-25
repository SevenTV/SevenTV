use std::net::SocketAddr;

use serde::Deserialize;
use shared::config::{DatabaseConfig, Http, HttpCors};

#[derive(Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Api {
	/// http options
	pub http: Http,
	/// public domain
	pub domain: String,
	/// base url
	pub base_url: String,
	/// cors options
	pub cors: HttpCors,
	/// connection config
	pub connections: ConnectionsConfig,
	/// jwt config
	pub jwt: JwtConfig,
}

impl Default for Api {
	fn default() -> Self {
		Self {
			http: Http::new_with_bind(SocketAddr::from(([0, 0, 0, 0], 8080))),
			domain: "7tv.io".to_string(),
			base_url: "https://7tv.io".to_string(),
			cors: HttpCors::default(),
			connections: ConnectionsConfig::default(),
			jwt: JwtConfig::default(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, config::Config, serde::Deserialize)]
#[serde(default)]
pub struct JwtConfig {
	/// JWT secret
	pub secret: String,

	/// JWT issuer
	pub issuer: String,
}

impl Default for JwtConfig {
	fn default() -> Self {
		Self {
			issuer: "seventv-api".to_string(),
			secret: "seventv-api".to_string(),
		}
	}
}

#[derive(Debug, Clone, Deserialize, config::Config)]
#[serde(default)]
pub struct ConnectionsConfig {
	/// Callback URL
	pub callback_url: String,
	/// Twitch connection
	pub twitch: ConnectionConfig,
	/// Discord connection
	pub discord: ConnectionConfig,
	/// Google connection
	pub google: ConnectionConfig,
}

impl Default for ConnectionsConfig {
	fn default() -> Self {
		Self {
			callback_url: "https://7tv.app/auth/callback".to_string(),
			twitch: ConnectionConfig::default(),
			discord: ConnectionConfig::default(),
			google: ConnectionConfig::default(),
		}
	}
}

#[derive(Debug, Clone, Deserialize, config::Config)]
pub struct ConnectionConfig {
	/// Client ID
	pub client_id: String,
	/// Client Secret
	pub client_secret: String,
}

impl Default for ConnectionConfig {
	fn default() -> Self {
		Self {
			client_id: "client_id".to_string(),
			client_secret: "client_secret".to_string(),
		}
	}
}

#[derive(Default, Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Extra {
	/// API configuration
	pub api: Api,

	/// Database configuration
	pub database: DatabaseConfig,
}

pub type Config = shared::config::Config<Extra>;
