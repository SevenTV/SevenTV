use std::net::SocketAddr;

use serde::Deserialize;
use shared::config::TlsConfig;

#[derive(Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Api {
	/// API bind
	pub bind: SocketAddr,
	/// Max Listen Conn
	pub listen_backlog: u32,
	/// TLS configuration
	pub tls: Option<TlsConfig>,
}

impl Default for Api {
	fn default() -> Self {
		Self {
			bind: SocketAddr::new([0, 0, 0, 0].into(), 3000),
			listen_backlog: 128,
			tls: None,
		}
	}
}

#[derive(Default, Debug, Deserialize, config::Config)]
#[serde(default)]
pub struct Extra {
	/// API configuration
	pub api: Api,
}

pub type Config = shared::config::Config<Extra>;
