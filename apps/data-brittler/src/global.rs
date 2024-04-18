use anyhow::Context as _;
use scuffle_utils::context::Context;

use crate::config::Config;

pub struct Global {
	ctx: Context,
	config: Config,
	clickhouse: clickhouse::Client,
	source_db: mongodb::Database,
	target_db: mongodb::Database,
}

impl Global {
	pub async fn new(ctx: Context, config: Config) -> anyhow::Result<Self> {
		let clickhouse = clickhouse::Client::default().with_url(&config.clickhouse);

		let mongo = shared::database::setup_database(&config.database)
			.await
			.context("database setup")?;

		let source_db = mongo.database(&config.source_db);
		let target_db = mongo.database(&config.target_db);

		Ok(Self {
			ctx,
			config,
			clickhouse,
			source_db,
			target_db,
		})
	}

	pub fn ctx(&self) -> &Context {
		&self.ctx
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
