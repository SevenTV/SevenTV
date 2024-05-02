use std::path::PathBuf;

use scuffle_foundations::settings::auto_settings;
use shared::config::DatabaseConfig;

pub type Config = shared::config::Config<Extra>;

#[auto_settings]
#[serde(default)]
pub struct Extra {
	/// Source database configuration
	#[settings(default = DatabaseConfig {
		uri: "mongodb://localhost:27017/7tv".to_string(),
		..Default::default()
	})]
	pub source_database: DatabaseConfig,
	/// Target database configuration
	#[settings(default = DatabaseConfig {
		uri: "mongodb://localhost:27017/7tv-new".to_string(),
		..Default::default()
	})]
	pub target_database: DatabaseConfig,
	/// ClickHouse connection string
	#[settings(default = DatabaseConfig {
		uri: "http://localhost:8123".to_string(),
		..Default::default()
	})]
	pub clickhouse: DatabaseConfig,
	/// Path to the report file
	#[settings(default = PathBuf::from("./local/report.md"))]
	pub report_path: PathBuf,

	/// Run users job
	pub users: bool,
	/// Skip users job
	pub skip_users: bool,

	/// Run bans job
	pub bans: bool,
	/// Skip bans job
	pub skip_bans: bool,

	/// Run emotes job
	pub emotes: bool,
	/// Skip emotes job
	pub skip_emotes: bool,

	/// Run emote sets job
	pub emote_sets: bool,
	/// Skip emote sets job
	pub skip_emote_sets: bool,

	/// Run cosmetics job
	pub cosmetics: bool,
	/// Skip cosmetics job
	pub skip_cosmetics: bool,

	/// Run roles job
	pub roles: bool,
	/// Skip roless job
	pub skip_roles: bool,

	/// Run reports job
	pub reports: bool,
	/// Skip reports job
	pub skip_reports: bool,

	/// Run audit logs job
	pub audit_logs: bool,
	/// Skip audit logs job
	pub skip_audit_logs: bool,

	/// Run messages job
	pub messages: bool,
	/// Skip messages job
	pub skip_messages: bool,

	/// Run system job
	pub system: bool,
	/// Skip system job
	pub skip_system: bool,

	/// Truncate tables before inserting data
	pub truncate: bool,
}
