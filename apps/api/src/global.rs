#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Context as _;
use bson::doc;
use scuffle_foundations::batcher::dataloader::DataLoader;
use scuffle_foundations::telemetry::server::HealthCheck;
use shared::database::badge::Badge;
use shared::database::emote::Emote;
use shared::database::emote_set::EmoteSet;
use shared::database::entitlement_edge::{EntitlementEdgeInboundLoader, EntitlementEdgeOutboundLoader};
use shared::database::global::GlobalConfig;
use shared::database::loader::LoaderById;
use shared::database::paint::Paint;
use shared::database::product::Product;
use shared::database::role::Role;
use shared::database::stored_event::StoredEvent;
use shared::database::ticket::Ticket;
use shared::database::user::ban::UserBan;
use shared::database::user::editor::UserEditor;
use shared::database::user::profile_picture::UserProfilePicture;
use shared::database::user::User;
use shared::image_processor::ImageProcessor;

use crate::config::Config;
use crate::dataloader::emote::EmoteByUserIdLoader;
use crate::dataloader::emote_set::EmoteSetByUserIdLoader;
use crate::dataloader::full_user::FullUserLoader;
use crate::dataloader::ticket_message::TicketMessageByTicketIdLoader;
use crate::dataloader::user::UserByPlatformIdLoader;
use crate::dataloader::user_bans::UserBanByUserIdLoader;
use crate::dataloader::user_editor::{UserEditorByEditorIdLoader, UserEditorByUserIdLoader};

pub struct Global {
	pub nats: async_nats::Client,
	pub jetstream: async_nats::jetstream::Context,
	pub config: Config,
	pub mongo: mongodb::Client,
	pub db: mongodb::Database,
	pub clickhouse: clickhouse::Client,
	pub http_client: reqwest::Client,
	pub image_processor: ImageProcessor,
	pub event_by_id_loader: DataLoader<LoaderById<StoredEvent>>,
	pub product_by_id_loader: DataLoader<LoaderById<Product>>,
	pub role_by_id_loader: DataLoader<LoaderById<Role>>,
	pub paint_by_id_loader: DataLoader<LoaderById<Paint>>,
	pub badge_by_id_loader: DataLoader<LoaderById<Badge>>,
	pub emote_by_id_loader: DataLoader<LoaderById<Emote>>,
	pub emote_by_user_id_loader: DataLoader<EmoteByUserIdLoader>,
	pub emote_set_by_id_loader: DataLoader<LoaderById<EmoteSet>>,
	pub emote_set_by_user_id_loader: DataLoader<EmoteSetByUserIdLoader>,
	pub global_config_loader: DataLoader<LoaderById<GlobalConfig>>,
	pub user_editor_by_user_id_loader: DataLoader<UserEditorByUserIdLoader>,
	pub user_editor_by_editor_id_loader: DataLoader<UserEditorByEditorIdLoader>,
	pub user_editor_by_id_loader: DataLoader<LoaderById<UserEditor>>,
	pub ticket_by_id_loader: DataLoader<LoaderById<Ticket>>,
	pub ticket_message_by_ticket_id_loader: DataLoader<TicketMessageByTicketIdLoader>,
	pub entitlement_edge_inbound_loader: DataLoader<EntitlementEdgeInboundLoader>,
	pub entitlement_edge_outbound_loader: DataLoader<EntitlementEdgeOutboundLoader>,
	pub user_by_id_loader: DataLoader<LoaderById<User>>,
	pub user_by_platform_id_loader: DataLoader<UserByPlatformIdLoader>,
	pub user_ban_by_id_loader: DataLoader<LoaderById<UserBan>>,
	pub user_ban_by_user_id_loader: DataLoader<UserBanByUserIdLoader>,
	pub user_profile_picture_id_loader: DataLoader<LoaderById<UserProfilePicture>>,
	pub user_loader: FullUserLoader,
	pub typesense: typesense_codegen::apis::configuration::Configuration,
}

impl Global {
	pub async fn new(config: Config) -> anyhow::Result<Arc<Self>> {
		let (nats, jetstream) = shared::nats::setup_nats("api", &config.nats).await.context("nats connect")?;
		let mongo = shared::database::setup_and_init_database(&config.database)
			.await
			.context("database setup")?;

		let db = mongo
			.default_database()
			.ok_or_else(|| anyhow::anyhow!("No default database"))?;

		let clickhouse = clickhouse::Client::default().with_url(&config.clickhouse.uri);

		let image_processor = ImageProcessor::new(&config.api.image_processor)
			.await
			.context("image processor setup")?;

		let typesense = typesense_codegen::apis::configuration::Configuration {
			base_path: config.typesense.uri.clone(),
			api_key: config
				.typesense
				.api_key
				.clone()
				.map(|key| typesense_codegen::apis::configuration::ApiKey { key, prefix: None }),
			..Default::default()
		};

		Ok(Arc::new_cyclic(|weak| Self {
			nats,
			jetstream,
			image_processor,
			event_by_id_loader: LoaderById::new(db.clone()),
			product_by_id_loader: LoaderById::new(db.clone()),
			role_by_id_loader: LoaderById::new(db.clone()),
			paint_by_id_loader: LoaderById::new(db.clone()),
			badge_by_id_loader: LoaderById::new(db.clone()),
			emote_by_id_loader: LoaderById::new(db.clone()),
			emote_by_user_id_loader: EmoteByUserIdLoader::new(db.clone()),
			emote_set_by_id_loader: LoaderById::new(db.clone()),
			emote_set_by_user_id_loader: EmoteSetByUserIdLoader::new(db.clone()),
			global_config_loader: LoaderById::new(db.clone()),
			user_editor_by_user_id_loader: UserEditorByUserIdLoader::new(db.clone()),
			user_editor_by_editor_id_loader: UserEditorByEditorIdLoader::new(db.clone()),
			user_editor_by_id_loader: LoaderById::new(db.clone()),
			ticket_by_id_loader: LoaderById::new(db.clone()),
			ticket_message_by_ticket_id_loader: TicketMessageByTicketIdLoader::new(db.clone()),
			entitlement_edge_inbound_loader: EntitlementEdgeInboundLoader::new(db.clone()),
			entitlement_edge_outbound_loader: EntitlementEdgeOutboundLoader::new(db.clone()),
			user_by_id_loader: LoaderById::new(db.clone()),
			user_by_platform_id_loader: UserByPlatformIdLoader::new(db.clone()),
			user_ban_by_id_loader: LoaderById::new(db.clone()),
			user_ban_by_user_id_loader: UserBanByUserIdLoader::new(db.clone()),
			user_profile_picture_id_loader: LoaderById::new(db.clone()),
			http_client: reqwest::Client::new(),
			typesense,
			mongo,
			db,
			clickhouse,
			config,
			user_loader: FullUserLoader::new(weak.clone()),
		}))
	}
}

impl HealthCheck for Global {
	fn check(&self) -> std::pin::Pin<Box<dyn futures::prelude::Future<Output = bool> + Send + '_>> {
		Box::pin(async {
			tracing::info!("running health check");

			if !match self.db.run_command(doc! { "ping": 1 }).await {
				Ok(r) => r.get_bool("ok").unwrap_or(false),
				Err(err) => {
					tracing::error!(%err, "failed to ping database");

					false
				}
			} {
				return false;
			}

			if !matches!(self.nats.connection_state(), async_nats::connection::State::Connected) {
				tracing::error!("nats not connected");
				return false;
			}

			true
		})
	}
}
