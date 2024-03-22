use std::net::SocketAddr;

use serde::Deserialize;
use shared::config::{DatabaseConfig, Http, HttpCors};

#[derive(Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Api {
	/// http options
	pub http: Http,
	/// cors options
	pub cors: HttpCors,
	/// connection config
	pub connections: Connections,
}

impl Default for Api {
	fn default() -> Self {
		Self {
			http: Http::new_with_bind(SocketAddr::from(([0, 0, 0, 0], 8080))),
			cors: HttpCors::default(),
			connections: Connections::default(),
		}
	}
}

#[derive(Debug, Default, Deserialize, config::Config)]
#[serde(default)]
pub struct Connections {
	/// Twitch connection
	pub twitch: Connection,
	/// Discord connection
	pub discord: Connection,
	/// Google connection
	pub google: Connection,
}

#[derive(Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Connection {
	/// Client ID
	pub client_id: String,
	/// Client Secret
	pub client_secret: String,
	/// Redirect URI
	pub redirect_uri: String,
}

impl Default for Connection {
	fn default() -> Self {
		Self {
			client_id: "client_id".to_string(),
			client_secret: "client_secret".to_string(),
			redirect_uri: "http://localhost:8080".to_string(),
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
