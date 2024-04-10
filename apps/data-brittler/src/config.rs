use std::path::PathBuf;

use shared::config::DatabaseConfig;

pub type Config = shared::config::Config<Extra>;

#[derive(Debug, serde::Deserialize, config::Config)]
#[serde(default)]
pub struct Extra {
	/// Database configuration
	pub database: DatabaseConfig,
	/// MongoDB connection string
	pub mongo: String,
	/// Path to the report file
	pub report_path: PathBuf,

	/// Run users job
	pub users: bool,
	/// Skip users job
	pub skip_users: bool,

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

	/// Truncate tables before inserting data
	pub truncate: bool,
}

impl Default for Extra {
	fn default() -> Self {
		Self {
			database: Default::default(),
			mongo: "mongodb://localhost:27017".to_string(),
			report_path: PathBuf::from("./local/report.md"),
			users: false,
			skip_users: false,
			emotes: false,
			skip_emotes: false,
			emote_sets: false,
			skip_emote_sets: false,
			cosmetics: false,
			skip_cosmetics: false,
			roles: false,
			skip_roles: false,
			truncate: false,
		}
	}
}
