use std::sync::Arc;

use anyhow::Context as _;
use scuffle_utils::context::Context;

use crate::config::Config;

pub struct Global {
	ctx: Context,
	config: Config,
	db: Arc<scuffle_utils::database::Pool>,
	mongo: mongodb::Client,
}

impl Global {
	pub async fn new(ctx: Context, config: Config) -> anyhow::Result<Self> {
		let db = shared::database::setup_database(&config.database)
			.await
			.context("database setup")?;

		let mut options = mongodb::options::ClientOptions::parse(&config.mongo).await?;
		options.app_name = Some("data-brittler".to_string());
		let mongo: mongodb::Client = mongodb::Client::with_options(options).context("failed to connect to MongoDB")?;

		Ok(Self { ctx, config, db, mongo })
	}

	pub fn ctx(&self) -> &Context {
		&self.ctx
	}

	pub fn config(&self) -> &Config {
		&self.config
	}

	pub fn db(&self) -> &Arc<scuffle_utils::database::Pool> {
		&self.db
	}

	pub fn mongo(&self) -> &mongodb::Client {
		&self.mongo
	}
}
