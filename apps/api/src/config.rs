use std::net::SocketAddr;

use scuffle_foundations::settings::auto_settings;
use shared::config::DatabaseConfig;

#[auto_settings]
#[serde(default)]
pub struct Api {
	/// http options
	#[settings(default = SocketAddr::from(([0, 0, 0, 0], 8080)))]
	pub bind: SocketAddr,
	/// worker count
	#[settings(default = 1)]
	pub workers: usize,
	/// cdn base url
	#[settings(default = "https://cdn.7tv.app".into())]
	pub cdn_base_url: String,
	/// public domain
	#[settings(default = "7tv.io".into())]
	pub domain: String,
	/// base url
	#[settings(default = "https://7tv.io".into())]
	pub base_url: String,
	/// cors options
	// pub cors: HttpCors,
	/// connection config
	pub connections: ConnectionsConfig,
	/// jwt config
	pub jwt: JwtConfig,
	/// image processor config
	pub image_processor: ImageProcessorConfig,
}

#[auto_settings]
#[serde(default)]
pub struct ImageProcessorConfig {
	/// Image Processor address
	pub address: Vec<String>,
	/// Resolve Interval
	#[settings(default = std::time::Duration::from_secs(10))]
	#[serde(with = "humantime_serde")]
	pub resolve_interval: std::time::Duration,
	/// Event Queue Name
	#[settings(default = "nats".into())]
	pub event_queue_name: String,
	/// Event Queue Topic Prefix
	#[settings(default = "image_processor".into())]
	pub event_queue_topic_prefix: String,
	/// Input Drive Name
	#[settings(default = "s3".into())]
	pub input_drive_name: String,
	/// Output Drive Name
	#[settings(default = "s3".into())]
	pub output_drive_name: String,
}

#[auto_settings]
#[serde(default)]
pub struct JwtConfig {
	/// JWT secret
	#[settings(default = "seventv-api".into())]
	pub secret: String,

	/// JWT issuer
	#[settings(default = "seventv-api".into())]
	pub issuer: String,
}

#[auto_settings]
#[serde(default)]
pub struct ConnectionsConfig {
	/// Callback URL
	#[settings(default = "https://7tv.app/auth/callback".into())]
	pub callback_url: String,
	/// Twitch connection
	pub twitch: ConnectionConfig,
	/// Discord connection
	pub discord: ConnectionConfig,
	/// Google connection
	pub google: ConnectionConfig,
}

#[auto_settings]
#[serde(default)]
pub struct ConnectionConfig {
	/// Client ID
	#[settings(default = "client_id".into())]
	pub client_id: String,
	/// Client Secret
	#[settings(default = "client_secret".into())]
	pub client_secret: String,
}

#[auto_settings]
#[serde(default)]
pub struct Extra {
	/// API configuration
	pub api: Api,

	/// Database configuration
	pub database: DatabaseConfig,
}

pub type Config = shared::config::Config<Extra>;
