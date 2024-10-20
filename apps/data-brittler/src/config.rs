use std::path::PathBuf;

use scuffle_foundations::bootstrap::Bootstrap;
use scuffle_foundations::settings::auto_settings;
use scuffle_foundations::telemetry::settings::{LoggingSettings, TelemetrySettings};
use shared::config::{ClickhouseConfig, DatabaseConfig, ImageProcessorConfig, NatsConfig};

#[auto_settings]
#[serde(default)]
pub struct Config {
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
	#[settings(default = ClickhouseConfig::default())]
	pub clickhouse: ClickhouseConfig,
	/// Path to the report file
	#[settings(default = PathBuf::from("./local/report.md"))]
	pub report_path: PathBuf,
	/// image processor config
	pub image_processor: ImageProcessorConfig,

	/// Only download cosmetics images, don't run jobs
	pub download_cosmetics: bool,

	/// Run users job
	pub users: Option<bool>,

	/// Copy over legacy user profile pictures
	pub legacy_user_pfps: Option<bool>,

	/// Run bans job
	pub bans: Option<bool>,

	/// Run emotes job
	pub emotes: Option<bool>,

	/// Run emote sets job
	pub emote_sets: Option<bool>,

	/// Run entitlments job
	pub entitlements: Option<bool>,

	/// Run cosmetics job
	pub cosmetics: Option<bool>,

	/// Run roles job
	pub roles: Option<bool>,

	/// Run reports job
	pub reports: Option<bool>,

	/// Run audit logs job
	pub audit_logs: Option<bool>,

	/// Run messages job
	pub messages: Option<bool>,

	/// Run system job
	pub system: Option<bool>,

	/// Run products job
	pub prices: Option<bool>,

	/// Create a list of files to copy for the new cdn
	pub cdn_rename: Option<bool>,

	/// Run subscriptions job
	pub subscriptions: Option<bool>,

	/// Run cron jobs
	pub cron_jobs: Option<bool>,

	/// Run special events job
	pub special_events: Option<bool>,

	/// Run redeem codes job
	pub redeem_codes: Option<bool>,

	/// Run emote stats job
	pub emote_stats: Option<bool>,

	/// Truncate tables before inserting data
	pub truncate: bool,

	/// NATs configuration
	pub nats: NatsConfig,

	/// Logging configuration
	pub logging: LoggingSettings,
}

impl Bootstrap for Config {
	type Settings = Self;

	fn telemetry_config(&self) -> Option<TelemetrySettings> {
		Some(TelemetrySettings {
			logging: self.logging.clone(),
			..Default::default()
		})
	}
}

impl Config {
	fn any_run(&self) -> bool {
		self.users.is_some_and(|r| r)
			|| self.bans.is_some_and(|r| r)
			|| self.emotes.is_some_and(|r| r)
			|| self.emote_sets.is_some_and(|r| r)
			|| self.entitlements.is_some_and(|r| r)
			|| self.cosmetics.is_some_and(|r| r)
			|| self.roles.is_some_and(|r| r)
			|| self.reports.is_some_and(|r| r)
			|| self.audit_logs.is_some_and(|r| r)
			|| self.messages.is_some_and(|r| r)
			|| self.system.is_some_and(|r| r)
			|| self.prices.is_some_and(|r| r)
			|| self.cdn_rename.is_some_and(|r| r)
			|| self.subscriptions.is_some_and(|r| r)
			|| self.special_events.is_some_and(|r| r)
			|| self.cron_jobs.is_some_and(|r| r)
			|| self.redeem_codes.is_some_and(|r| r)
			|| self.emote_stats.is_some_and(|r| r)
	}

	pub fn should_run_users(&self) -> bool {
		self.users.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_bans(&self) -> bool {
		self.bans.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_emotes(&self) -> bool {
		self.emotes.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_cdn_rename(&self) -> bool {
		self.cdn_rename.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_emote_sets(&self) -> bool {
		self.emote_sets.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_entitlements(&self) -> bool {
		self.entitlements.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_cosmetics(&self) -> bool {
		self.cosmetics.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_roles(&self) -> bool {
		self.roles.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_reports(&self) -> bool {
		self.reports.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_audit_logs(&self) -> bool {
		self.audit_logs.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_messages(&self) -> bool {
		self.messages.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_system(&self) -> bool {
		self.system.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_prices(&self) -> bool {
		self.prices.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_subscriptions(&self) -> bool {
		self.subscriptions.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_cron_jobs(&self) -> bool {
		self.cron_jobs.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_special_events(&self) -> bool {
		self.special_events.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_redeem_codes(&self) -> bool {
		self.redeem_codes.unwrap_or_else(|| !self.any_run())
	}

	pub fn should_run_emote_stats(&self) -> bool {
		self.emote_stats.unwrap_or_else(|| !self.any_run())
	}
}
