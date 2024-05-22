use anyhow::Context as _;

use crate::config::Config;

pub struct Global {
	config: Config,
	clickhouse: clickhouse::Client,
	stripe_client: stripe::Client,
	main_source_db: mongodb::Database,
	egvault_source_db: mongodb::Database,
	target_db: mongodb::Database,
}

impl Global {
	pub async fn new(config: Config) -> anyhow::Result<Self> {
		let clickhouse = clickhouse::Client::default().with_url(&config.clickhouse.uri);

		let mongo_source = shared::database::setup_database(&config.main_source_database)
			.await
			.context("source database setup")?;

		let mongo_egvault_source = shared::database::setup_database(&config.egvault_source_database)
			.await
			.context("egvault source database setup")?;

		let mongo_target = shared::database::setup_database(&config.target_database)
			.await
			.context("target database setup")?;

		Ok(Self {
			stripe_client: stripe::Client::new(&config.stripe_key),
			config,
			clickhouse,
			main_source_db: mongo_source
				.default_database()
				.unwrap_or_else(|| mongo_source.database("7tv")),
			egvault_source_db: mongo_egvault_source
				.default_database()
				.unwrap_or_else(|| mongo_egvault_source.database("egvault")),
			target_db: mongo_target
				.default_database()
				.unwrap_or_else(|| mongo_target.database("7tv-new")),
		})
	}

	pub fn config(&self) -> &Config {
		&self.config
	}

	pub fn clickhouse(&self) -> &clickhouse::Client {
		&self.clickhouse
	}

	pub fn source_db(&self) -> &mongodb::Database {
		&self.main_source_db
	}

	pub fn egvault_source_db(&self) -> &mongodb::Database {
		&self.egvault_source_db
	}

	pub fn target_db(&self) -> &mongodb::Database {
		&self.target_db
	}

	pub fn stripe_client(&self) -> &stripe::Client {
		&self.stripe_client
	}
}
