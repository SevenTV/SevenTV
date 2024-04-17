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
	mongo: mongodb::Client,
	db: mongodb::Database,
	http_client: reqwest::Client,
	metrics: Arc<metrics::Metrics>,
	user_by_id_loader: dataloader::user::UserLoader,
	user_connection_by_user_id_loader: DataLoader<dataloader::user_connection::UserConnectionByUserIdLoader>,
	product_by_id_loader: DataLoader<dataloader::product::ProductByIdLoader>,
	role_by_id_loader: DataLoader<dataloader::role::RoleByIdLoader>,
	file_set_by_id_loader: DataLoader<dataloader::file_set::FileSetByIdLoader>,
	paint_by_id_loader: DataLoader<dataloader::paint::PaintByIdLoader>,
	badge_by_id_loader: DataLoader<dataloader::badge::BadgeByIdLoader>,
	emote_by_id_loader: DataLoader<dataloader::emote::EmoteByIdLoader>,
	emote_set_by_id_loader: DataLoader<dataloader::emote_set::EmoteSetByIdLoader>,
	emote_set_emote_by_id_loader: DataLoader<dataloader::emote_set::EmoteSetEmoteByIdLoader>,
	emote_set_by_user_id_loader: DataLoader<dataloader::emote_set::EmoteSetByUserIdLoader>,
	global_config_loader: DataLoader<dataloader::global_config::GlobalConfigLoader>,
	user_editor_by_user_id_loader: DataLoader<dataloader::user_editor::UserEditorByUserIdLoader>,
	user_editor_by_editor_id_loader: DataLoader<dataloader::user_editor::UserEditorByEditorIdLoader>,
}

impl Global {
	pub async fn new(ctx: Context, config: Config) -> anyhow::Result<Self> {
		let (nats, jetstream) = shared::nats::setup_nats("api", &config.nats).await.context("nats connect")?;
		let mongo = shared::database::setup_database(&config.database)
			.await
			.context("database setup")?;

		let db = mongo.default_database().unwrap_or_else(|| mongo.database("7tv"));

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
			user_connection_by_user_id_loader: dataloader::user_connection::UserConnectionByUserIdLoader::new(db.clone()),
			product_by_id_loader: dataloader::product::ProductByIdLoader::new(db.clone()),
			role_by_id_loader: dataloader::role::RoleByIdLoader::new(db.clone()),
			file_set_by_id_loader: dataloader::file_set::FileSetByIdLoader::new(db.clone()),
			paint_by_id_loader: dataloader::paint::PaintByIdLoader::new(db.clone()),
			badge_by_id_loader: dataloader::badge::BadgeByIdLoader::new(db.clone()),
			emote_by_id_loader: dataloader::emote::EmoteByIdLoader::new(db.clone()),
			emote_set_by_id_loader: dataloader::emote_set::EmoteSetByIdLoader::new(db.clone()),
			emote_set_emote_by_id_loader: dataloader::emote_set::EmoteSetEmoteByIdLoader::new(db.clone()),
			emote_set_by_user_id_loader: dataloader::emote_set::EmoteSetByUserIdLoader::new(db.clone()),
			global_config_loader: dataloader::global_config::GlobalConfigLoader::new(db.clone()),
			user_editor_by_user_id_loader: dataloader::user_editor::UserEditorByUserIdLoader::new(db.clone()),
			user_editor_by_editor_id_loader: dataloader::user_editor::UserEditorByEditorIdLoader::new(db.clone()),
			http_client: reqwest::Client::new(),
			mongo,
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

	/// The MongoDB database.
	pub fn db(&self) -> &mongodb::Database {
		&self.db
	}

	/// The MongoDB client.
	pub fn mongo(&self) -> &mongodb::Client {
		&self.mongo
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
	pub fn user_connection_by_user_id_loader(
		&self,
	) -> &DataLoader<dataloader::user_connection::UserConnectionByUserIdLoader> {
		&self.user_connection_by_user_id_loader
	}

	/// The product loader.
	pub fn product_by_id_loader(&self) -> &DataLoader<dataloader::product::ProductByIdLoader> {
		&self.product_by_id_loader
	}

	/// The role loader.
	pub fn role_by_id_loader(&self) -> &DataLoader<dataloader::role::RoleByIdLoader> {
		&self.role_by_id_loader
	}

	/// The file loader.
	pub fn file_set_by_id_loader(&self) -> &DataLoader<dataloader::file_set::FileSetByIdLoader> {
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

	/// The emote set loader.
	pub fn emote_set_by_id_loader(&self) -> &DataLoader<dataloader::emote_set::EmoteSetByIdLoader> {
		&self.emote_set_by_id_loader
	}

	/// The emote set emote loader.
	pub fn emote_set_emote_by_id_loader(&self) -> &DataLoader<dataloader::emote_set::EmoteSetEmoteByIdLoader> {
		&self.emote_set_emote_by_id_loader
	}

	/// The emote set by user loader.
	pub fn emote_set_by_user_id_loader(&self) -> &DataLoader<dataloader::emote_set::EmoteSetByUserIdLoader> {
		&self.emote_set_by_user_id_loader
	}

	/// The global config loader.
	pub fn global_config_loader(&self) -> &DataLoader<dataloader::global_config::GlobalConfigLoader> {
		&self.global_config_loader
	}

	/// The user editor by user loader.
	pub fn user_editor_by_user_id_loader(&self) -> &DataLoader<dataloader::user_editor::UserEditorByUserIdLoader> {
		&self.user_editor_by_user_id_loader
	}

	/// The user editor by editor loader.
	pub fn user_editor_by_editor_id_loader(&self) -> &DataLoader<dataloader::user_editor::UserEditorByEditorIdLoader> {
		&self.user_editor_by_editor_id_loader
	}
}
