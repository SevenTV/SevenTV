#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Context as _;
use scuffle_utils::context::Context;
use scuffle_utils::dataloader::DataLoader;

use crate::config::Config;
use crate::{dataloader, metrics};

pub struct Global {
	ctx: Context,
	nats: async_nats::Client,
	jetstream: async_nats::jetstream::Context,
	config: Config,
	db: Arc<scuffle_utils::database::Pool>,
	http_client: reqwest::Client,
	metrics: Arc<metrics::Metrics>,
	user_by_id_loader: dataloader::user::UserLoader,
	user_connections_loader: DataLoader<dataloader::user_connections::UserConnectionsByUserIdLoader>,
	product_by_id_loader: DataLoader<dataloader::product::ProductByIdLoader>,
	role_by_id_loader: DataLoader<dataloader::role::RoleByIdLoader>,
	role_badge_by_id_loader: DataLoader<dataloader::role::RoleBadgeByIdLoader>,
	role_paint_by_id_loader: DataLoader<dataloader::role::RolePaintByIdLoader>,
	role_emote_set_by_id_loader: DataLoader<dataloader::role::RoleEmoteSetByIdLoader>,
	file_set_by_id_loader: DataLoader<dataloader::file::FileSetByIdLoader>,
	paint_by_id_loader: DataLoader<dataloader::paint::PaintByIdLoader>,
	badge_by_id_loader: DataLoader<dataloader::badge::BadgeByIdLoader>,
	emote_by_id_loader: DataLoader<dataloader::emote::EmoteByIdLoader>,
}

impl Global {
	pub async fn new(ctx: Context, config: Config) -> anyhow::Result<Self> {
		let (nats, jetstream) = shared::nats::setup_nats("api", &config.nats).await.context("nats connect")?;
		let db = shared::database::setup_database(&config.database)
			.await
			.context("database setup")?;

		Ok(Self {
			metrics: Arc::new(metrics::new(
				config
					.metrics
					.labels
					.iter()
					.map(|x| (x.key.clone(), x.value.clone()))
					.collect(),
			)),
			ctx,
			nats,
			jetstream,
			user_by_id_loader: dataloader::user::UserLoader::new(db.clone()),
			user_connections_loader: dataloader::user_connections::UserConnectionsByUserIdLoader::new(db.clone()),
			product_by_id_loader: dataloader::product::ProductByIdLoader::new(db.clone()),
			role_by_id_loader: dataloader::role::RoleByIdLoader::new(db.clone()),
			role_badge_by_id_loader: dataloader::role::RoleBadgeByIdLoader::new(db.clone()),
			role_paint_by_id_loader: dataloader::role::RolePaintByIdLoader::new(db.clone()),
			role_emote_set_by_id_loader: dataloader::role::RoleEmoteSetByIdLoader::new(db.clone()),
			file_set_by_id_loader: dataloader::file::FileSetByIdLoader::new(db.clone()),
			paint_by_id_loader: dataloader::paint::PaintByIdLoader::new(db.clone()),
			badge_by_id_loader: dataloader::badge::BadgeByIdLoader::new(db.clone()),
			emote_by_id_loader: dataloader::emote::EmoteByIdLoader::new(db.clone()),
			http_client: reqwest::Client::new(),
			db,
			config,
		})
	}

	/// The global context.
	pub fn ctx(&self) -> &Context {
		&self.ctx
	}

	/// The NATS client.
	pub fn nats(&self) -> &async_nats::Client {
		&self.nats
	}

	/// The NATS JetStream context.
	pub fn jetstream(&self) -> &async_nats::jetstream::Context {
		&self.jetstream
	}

	/// The database pool.
	pub fn db(&self) -> &Arc<scuffle_utils::database::Pool> {
		&self.db
	}

	/// The configuration.
	pub fn config(&self) -> &Config {
		&self.config
	}

	/// Global HTTP client.
	pub fn http_client(&self) -> &reqwest::Client {
		&self.http_client
	}

	/// Global metrics.
	pub fn metrics(&self) -> &Arc<metrics::Metrics> {
		&self.metrics
	}

	/// The user loader.
	pub fn user_by_id_loader(&self) -> &dataloader::user::UserLoader {
		&self.user_by_id_loader
	}

	/// The user connections loader.
	pub fn user_connections_loader(&self) -> &DataLoader<dataloader::user_connections::UserConnectionsByUserIdLoader> {
		&self.user_connections_loader
	}

	/// The product loader.
	pub fn product_by_id_loader(&self) -> &DataLoader<dataloader::product::ProductByIdLoader> {
		&self.product_by_id_loader
	}

	/// The role loader.
	pub fn role_by_id_loader(&self) -> &DataLoader<dataloader::role::RoleByIdLoader> {
		&self.role_by_id_loader
	}

	/// The role badge loader.
	pub fn role_badge_by_id_loader(&self) -> &DataLoader<dataloader::role::RoleBadgeByIdLoader> {
		&self.role_badge_by_id_loader
	}

	/// The role paint loader.
	pub fn role_paint_by_id_loader(&self) -> &DataLoader<dataloader::role::RolePaintByIdLoader> {
		&self.role_paint_by_id_loader
	}

	/// The role emote set loader.
	pub fn role_emote_set_by_id_loader(&self) -> &DataLoader<dataloader::role::RoleEmoteSetByIdLoader> {
		&self.role_emote_set_by_id_loader
	}

	/// The file loader.
	pub fn file_set_by_id_loader(&self) -> &DataLoader<dataloader::file::FileSetByIdLoader> {
		&self.file_set_by_id_loader
	}

	/// The paint loader.
	pub fn paint_by_id_loader(&self) -> &DataLoader<dataloader::paint::PaintByIdLoader> {
		&self.paint_by_id_loader
	}

	/// The badge loader.
	pub fn badge_by_id_loader(&self) -> &DataLoader<dataloader::badge::BadgeByIdLoader> {
		&self.badge_by_id_loader
	}

	/// The emote loader.
	pub fn emote_by_id_loader(&self) -> &DataLoader<dataloader::emote::EmoteByIdLoader> {
		&self.emote_by_id_loader
	}
}
