use std::collections::HashMap;

use anyhow::Context as _;
use scuffle_image_processor_proto::EventCallback;
use shared::image_processor::ImageProcessor;
use tokio::sync::Mutex;

use crate::config::Config;

pub struct Global {
	pub config: Config,
	pub clickhouse: clickhouse::Client,
	pub main_source_db: mongodb::Database,
	pub egvault_source_db: mongodb::Database,
	pub target_db: mongodb::Database,
	pub http_client: reqwest::Client,
	pub image_processor: ImageProcessor,
	pub jetstream: async_nats::jetstream::Context,
	pub all_tasks: Mutex<HashMap<String, tokio::sync::mpsc::Sender<EventCallback>>>,
}

impl Global {
	pub async fn new(config: Config) -> anyhow::Result<Self> {
		let clickhouse = shared::clickhouse::init_clickhouse(&config.clickhouse).await?;

		let mongo_source = shared::database::setup_database(&config.main_source_database, false)
			.await
			.context("source database setup")?;

		tracing::info!("source database setup complete");

		let mongo_egvault_source = shared::database::setup_database(&config.egvault_source_database, false)
			.await
			.context("egvault source database setup")?;

		tracing::info!("egvault source database setup complete");

		let mongo_target = shared::database::setup_and_init_database(&config.target_database)
			.await
			.context("target database setup")?;

		tracing::info!("target database setup complete");

		let image_processor = ImageProcessor::new(&config.image_processor).await?;

		tracing::info!("image processor setup complete");

		let (_, jetstream) = shared::nats::setup_nats("api", &config.nats).await.context("nats connect")?;

		tracing::info!("nats setup complete");

		Ok(Self {
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
			jetstream,
			all_tasks: Default::default(),
		})
	}
}
