use std::net::SocketAddr;
use std::path::PathBuf;

use shared::config::{
	ClickhouseConfig, DatabaseConfig, ImageProcessorConfig, IncomingRequestConfig, NatsConfig, RedisConfig, TypesenseConfig,
};
use shared::ip::GeoIpConfig;

#[derive(Debug, Clone, smart_default::SmartDefault, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Api {
	/// http options
	#[default(SocketAddr::from(([0, 0, 0, 0], 8080)))]
	pub bind: SocketAddr,

	/// worker count
	#[default(1)]
	pub workers: usize,

	/// website origin
	#[default("https://7tv.app".parse().unwrap())]
	pub website_origin: url::Url,

	/// beta website origin
	#[default("https://beta.7tv.app".parse().unwrap())]
	pub beta_website_origin: url::Url,

	/// cdn base url
	#[default("https://cdn.7tv.app/".parse().unwrap())]
	pub cdn_origin: url::Url,

	/// public domain
	#[default("7tv.io".into())]
	pub domain: String,

	/// base url
	#[default("https://7tv.io".parse().unwrap())]
	pub api_origin: url::Url,

	/// All orgins which are allowed to send CORS requests with credentials
	/// included
	#[default(vec!["https://twitch.tv".parse().unwrap(), "https://kick.com".parse().unwrap()])]
	pub cors_allowed_credential_origins: Vec<url::Url>,

	/// Event API nats prefix
	#[default("api.events".into())]
	pub nats_event_subject: String,

	/// IP Header config
	pub incoming_request: IncomingRequestConfig,
}

#[derive(Debug, Clone, smart_default::SmartDefault, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct JwtConfig {
	/// JWT secret
	#[default("seventv-api".into())]
	pub secret: String,

	/// JWT issuer
	#[default("seventv-api".into())]
	pub issuer: String,
}

#[derive(Debug, Clone, smart_default::SmartDefault, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ConnectionsConfig {
	/// Twitch connection
	pub twitch: ConnectionConfig,
	/// Discord connection
	pub discord: ConnectionConfig,
	/// Google connection
	pub google: ConnectionConfig,
}

#[derive(Debug, Clone, smart_default::SmartDefault, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ConnectionConfig {
	/// If login with this connection is enabled
	#[default(false)]
	pub enabled: bool,
	/// Client ID
	#[default("client_id".into())]
	pub client_id: String,
	/// Client Secret
	#[default("client_secret".into())]
	pub client_secret: String,
}

#[derive(Debug, Clone, smart_default::SmartDefault, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ChangeStreamConfig {
	/// Change Stream Prefix
	#[default("seventv".into())]
	pub prefix: String,

	/// The number of pending acks to buffer
	#[default(1000)]
	pub back_pressure_limit: usize,
}

#[derive(Debug, Clone, smart_default::SmartDefault, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct StripeConfig {
	/// Stripe API key
	#[default("sk_test_123".into())]
	pub api_key: String,
	/// Stripe webhook secret
	#[default("whsec_test".into())]
	pub webhook_secret: String,
	/// Stripe concurrent requests
	#[default(50)]
	pub concurrent_requests: usize,
}

#[derive(Debug, Clone, smart_default::SmartDefault, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PayPalConfig {
	/// Paypal client id
	#[default("client_id".into())]
	pub client_id: String,
	/// Paypal client secret
	#[default("client_secret".into())]
	pub client_secret: String,
	/// PayPal webhook id
	#[default("webhook_id".into())]
	pub webhook_id: String,
}

#[derive(Debug, Clone, smart_default::SmartDefault, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Config {
	/// Export GQL schema
	pub export_schema_path: Option<PathBuf>,

	/// API configuration
	pub api: Api,

	/// Database configuration
	pub database: DatabaseConfig,

	/// Clickhouse configuration
	pub clickhouse: ClickhouseConfig,

	/// NATs configuration
	pub nats: NatsConfig,

	/// Typesense configuration
	pub typesense: TypesenseConfig,

	/// jwt config
	pub jwt: JwtConfig,

	/// image processor config
	pub image_processor: ImageProcessorConfig,

	/// connection config
	pub connections: ConnectionsConfig,

	/// Redis config
	pub redis: RedisConfig,

	/// Stripe config
	pub stripe: StripeConfig,

	/// PayPal config
	pub paypal: PayPalConfig,

	/// GeoIP config
	pub geoip: Option<GeoIpConfig>,

	/// CDN purge topic
	pub cdn: CdnConfig,

	/// Log level
	#[default(std::env::var("RUST_LOG").unwrap_or("info".into()))]
	pub level: String,

	/// Metrics bind address
	#[default(None)]
	pub metrics_bind_address: Option<SocketAddr>,
}

#[derive(Debug, Clone, smart_default::SmartDefault, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CdnConfig {
	/// CDN purge stream name
	#[default("CdnPurge".into())]
	pub purge_stream_name: String,

	/// CDN purge stream subject
	#[default("cdn.purge".into())]
	pub purge_stream_subject: String,

	/// Cloudflare CDN zone id
	#[default("".into())]
	pub cloudflare_cdn_zone_id: String,

	/// Cloudflare API token
	#[default("".into())]
	pub cloudflare_api_token: String,
}

scuffle_bootstrap::cli_config!(Config);
