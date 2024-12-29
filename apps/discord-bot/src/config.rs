use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use shared::config::{DatabaseConfig, NatsConfig};

#[derive(Debug, Serialize, Deserialize, smart_default::SmartDefault)]
#[serde(default)]
pub struct Config {
	/// Bot configuration
	pub bot: Bot,

	/// Database configuration
	pub database: DatabaseConfig,

	/// NATS configuration
	pub nats: NatsConfig,

	/// Log level
	#[default(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))]
	pub level: String,

	/// Metrics bind address
	#[default(None)]
	pub metrics_bind_address: Option<SocketAddr>,
}

#[derive(Debug, Serialize, Deserialize, smart_default::SmartDefault)]
#[serde(default)]
pub struct Bot {
	/// Discord token
	pub discord_token: String,

	/// Guild ID
	#[default(817075418054000661)]
	pub guild_id: u64,

	/// Activity feed channel ID
	#[default(817375925271527449)]
	pub activity_feed_channel_id: u64,

	/// Contributor role ID
	#[default(822343588511219772)]
	pub contributor_role_id: u64,

	/// Subscriber role ID
	#[default(855366628555882497)]
	pub subscriber_role_id: u64,
}

scuffle_settings::bootstrap!(Config);
