use std::collections::HashSet;

use anyhow::Context as _;
use shared::image_processor::ImageProcessor;

use crate::config::Config;

pub struct Global {
	config: Config,
	clickhouse: clickhouse::Client,
	stripe_client: stripe::Client,
	main_source_db: mongodb::Database,
	egvault_source_db: mongodb::Database,
	target_db: mongodb::Database,
	http_client: reqwest::Client,
	image_processor: ImageProcessor,
	_nats: async_nats::Client,
	jetstream: async_nats::jetstream::Context,
	all_tasks: tokio::sync::OnceCell<HashSet<String>>,
	users_job_token: tokio_util::sync::CancellationToken,
	entitlement_job_token: tokio_util::sync::CancellationToken,
}

impl Global {
	pub async fn new(config: Config) -> anyhow::Result<Self> {
		let clickhouse = shared::clickhouse::init_clickhouse(&config.clickhouse).await?;

		let mongo_source = shared::database::setup_database(&config.main_source_database, false)
			.await
			.context("source database setup")?;

		let mongo_egvault_source = shared::database::setup_database(&config.egvault_source_database, false)
			.await
			.context("egvault source database setup")?;

		let mongo_target = shared::database::setup_and_init_database(&config.target_database)
			.await
			.context("target database setup")?;

		let image_processor = ImageProcessor::new(&config.image_processor).await?;

		let (nats, jetstream) = shared::nats::setup_nats("api", &config.nats).await.context("nats connect")?;

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
			http_client: reqwest::Client::new(),
			image_processor,
			_nats: nats,
			jetstream,
			all_tasks: tokio::sync::OnceCell::new(),
			users_job_token: tokio_util::sync::CancellationToken::new(),
			entitlement_job_token: tokio_util::sync::CancellationToken::new(),
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

	pub fn http_client(&self) -> &reqwest::Client {
		&self.http_client
	}

	pub fn stripe_client(&self) -> &stripe::Client {
		&self.stripe_client
	}

	pub fn image_processor(&self) -> &ImageProcessor {
		&self.image_processor
	}

	pub fn jetstream(&self) -> &async_nats::jetstream::Context {
		&self.jetstream
	}

	pub fn all_tasks(&self) -> &tokio::sync::OnceCell<HashSet<String>> {
		&self.all_tasks
	}

	pub fn users_job_token(&self) -> &tokio_util::sync::CancellationToken {
		&self.users_job_token
	}

	pub fn entitlement_job_token(&self) -> &tokio_util::sync::CancellationToken {
		&self.entitlement_job_token
	}
}
