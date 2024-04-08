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
	/// Skip user job
	pub skip_users: bool,
	/// Skip emote job
	pub skip_emotes: bool,
	/// Skip cosmetic job
	pub skip_cosmetics: bool,
	/// Truncate tables before inserting data
	pub truncate: bool,
}

impl Default for Extra {
	fn default() -> Self {
		Self {
			database: Default::default(),
			mongo: "mongodb://localhost:27017".to_string(),
			report_path: PathBuf::from("./local/report.md"),
			skip_users: false,
			skip_emotes: false,
			skip_cosmetics: false,
			truncate: false,
		}
	}
}
