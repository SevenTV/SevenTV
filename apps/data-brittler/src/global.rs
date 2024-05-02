use anyhow::Context as _;

use crate::config::Config;

pub struct Global {
	config: Config,
	clickhouse: clickhouse::Client,
	source_db: mongodb::Database,
	target_db: mongodb::Database,
}

impl Global {
	pub async fn new(config: Config) -> anyhow::Result<Self> {
		let clickhouse = clickhouse::Client::default().with_url(&config.clickhouse.uri);

		let mongo_source = shared::database::setup_database(&config.source_database)
			.await
			.context("source database setup")?;

		let mongo_target = shared::database::setup_database(&config.target_database)
			.await
			.context("target database setup")?;

		Ok(Self {
			config,
			clickhouse,
			source_db: mongo_source
				.default_database()
				.unwrap_or_else(|| mongo_source.database("7tv")),
			target_db: mongo_target
				.default_database()
				.unwrap_or_else(|| mongo_source.database("7tv-new")),
		})
	}

	pub fn config(&self) -> &Config {
		&self.config
	}

	pub fn clickhouse(&self) -> &clickhouse::Client {
		&self.clickhouse
	}

	pub fn source_db(&self) -> &mongodb::Database {
		&self.source_db
	}

	pub fn target_db(&self) -> &mongodb::Database {
		&self.target_db
	}
}
