use std::path::PathBuf;

use shared::config::DatabaseConfig;

pub type Config = shared::config::Config<Extra>;

#[derive(Debug, serde::Deserialize, config::Config)]
#[serde(default)]
pub struct Extra {
	/// Database configuration
	pub database: DatabaseConfig,
	/// ClickHouse connection string
	pub clickhouse: String,
	/// MongoDB connection string
	pub mongo: String,
	/// Path to the report file
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

	/// Truncate tables before inserting data
	pub truncate: bool,
}

impl Default for Extra {
	fn default() -> Self {
		Self {
			database: Default::default(),
			clickhouse: "http://localhost:8123".to_string(),
			mongo: "mongodb://localhost:27017".to_string(),
			report_path: PathBuf::from("./local/report.md"),
			users: false,
			skip_users: false,
			bans: false,
			skip_bans: false,
			emotes: false,
			skip_emotes: false,
			emote_sets: false,
			skip_emote_sets: false,
			cosmetics: false,
			skip_cosmetics: false,
			roles: false,
			skip_roles: false,
			reports: false,
			skip_reports: false,
			audit_logs: false,
			skip_audit_logs: false,
			truncate: false,
		}
	}
}
