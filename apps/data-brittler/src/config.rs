use std::path::PathBuf;

use scuffle_foundations::settings::auto_settings;
use shared::config::{DatabaseConfig, ImageProcessorConfig};

pub type Config = shared::config::Config<Extra>;

#[auto_settings]
#[serde(default)]
pub struct Extra {
	/// Main source database configuration
	#[settings(default = DatabaseConfig {
		uri: "mongodb://localhost:27017/7tv".to_string(),
	})]
	pub main_source_database: DatabaseConfig,
	/// Egvault source database configuration
	#[settings(default = DatabaseConfig {
		uri: "mongodb://localhost:27017/egvault".to_string(),
	})]
	pub egvault_source_database: DatabaseConfig,
	/// Target database configuration
	#[settings(default = DatabaseConfig {
		uri: "mongodb://localhost:27017/7tv-new".to_string(),
	})]
	pub target_database: DatabaseConfig,
	/// ClickHouse connection string
	#[settings(default = DatabaseConfig {
		uri: "http://localhost:8123".to_string(),
	})]
	pub clickhouse: DatabaseConfig,
	/// Path to the report file
	#[settings(default = PathBuf::from("./local/report.md"))]
	pub report_path: PathBuf,
	/// Stripe API key
	pub stripe_key: String,
	/// image processor config
	pub image_processor: ImageProcessorConfig,

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

	/// Run entitlments job
	pub entitlements: bool,
	/// Skip entitlements job
	pub skip_entitlements: bool,

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

	/// Run products job
	pub prices: bool,
	/// Skip products job
	pub skip_prices: bool,

	/// Run subscriptions job
	pub subscriptions: bool,
	/// Skip subscriptions job
	pub skip_subscriptions: bool,

	/// Truncate tables before inserting data
	pub truncate: bool,
}

impl Extra {
	fn any_run(&self) -> bool {
		self.users
			|| self.bans || self.emotes
			|| self.emote_sets
			|| self.entitlements
			|| self.cosmetics
			|| self.roles
			|| self.reports
			|| self.audit_logs
			|| self.messages
			|| self.system
			|| self.prices
			|| self.subscriptions
	}

	pub fn should_run_users(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.users || !any_run && !self.skip_users
	}

	pub fn should_run_bans(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.bans || !any_run && !self.skip_bans
	}

	pub fn should_run_emotes(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.emotes || !any_run && !self.skip_emotes
	}

	pub fn should_run_emote_sets(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.emote_sets || !any_run && !self.skip_emote_sets
	}

	pub fn should_run_entitlements(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.entitlements || !any_run && !self.skip_entitlements
	}

	pub fn should_run_cosmetics(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.cosmetics || !any_run && !self.skip_cosmetics
	}

	pub fn should_run_roles(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.roles || !any_run && !self.skip_roles
	}

	pub fn should_run_reports(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.reports || !any_run && !self.skip_reports
	}

	pub fn should_run_audit_logs(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.audit_logs || !any_run && !self.skip_audit_logs
	}

	pub fn should_run_messages(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.messages || !any_run && !self.skip_messages
	}

	pub fn should_run_system(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.system || !any_run && !self.skip_system
	}

	pub fn should_run_prices(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.prices || !any_run && !self.skip_prices
	}

	pub fn should_run_subscriptions(&self) -> bool {
		let any_run = self.any_run();
		any_run && self.subscriptions || !any_run && !self.skip_subscriptions
	}
}
