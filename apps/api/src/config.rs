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
}

impl Default for Api {
	fn default() -> Self {
		Self {
			http: Http::new_with_bind(SocketAddr::from(([0, 0, 0, 0], 8080))),
			cors: HttpCors::default(),
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
