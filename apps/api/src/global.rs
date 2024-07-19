#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Context as _;
use bson::doc;
use scuffle_foundations::batcher::dataloader::DataLoader;
use scuffle_foundations::telemetry::server::HealthCheck;
use shared::database::audit_log::AuditLog;
use shared::database::badge::Badge;
use shared::database::emote::Emote;
use shared::database::emote_set::EmoteSet;
use shared::database::entitlement_edge::{EntitlementEdgeInboundLoader, EntitlementEdgeOutboundLoader};
use shared::database::global::GlobalConfig;
use shared::database::loader::LoaderById;
use shared::database::paint::Paint;
use shared::database::product::Product;
use shared::database::role::Role;
use shared::database::ticket::Ticket;
use shared::database::user::ban::UserBan;
use shared::database::user::editor::UserEditor;
use shared::database::user::User;
use shared::image_processor::ImageProcessor;

use crate::config::Config;
use crate::dataloader::emote::EmoteByUserIdLoader;
use crate::dataloader::emote_set::EmoteSetByUserIdLoader;
use crate::dataloader::full_user::FullUserLoader;
use crate::dataloader::user::UserByPlatformIdLoader;
use crate::dataloader::user_bans::UserBanByUserIdLoader;
use crate::dataloader::user_editor::{UserEditorByEditorIdLoader, UserEditorByUserIdLoader};
use crate::event_api::EventApi;

pub struct Global {
	nats: async_nats::Client,
	jetstream: async_nats::jetstream::Context,
	config: Config,
	mongo: mongodb::Client,
	db: mongodb::Database,
	clickhouse: clickhouse::Client,
	http_client: reqwest::Client,
	event_api: EventApi,
	image_processor: ImageProcessor,
	audit_log_by_id_loader: DataLoader<LoaderById<AuditLog>>,
	product_by_id_loader: DataLoader<LoaderById<Product>>,
	role_by_id_loader: DataLoader<LoaderById<Role>>,
	paint_by_id_loader: DataLoader<LoaderById<Paint>>,
	badge_by_id_loader: DataLoader<LoaderById<Badge>>,
	emote_by_id_loader: DataLoader<LoaderById<Emote>>,
	emote_by_user_id_loader: DataLoader<EmoteByUserIdLoader>,
	emote_set_by_id_loader: DataLoader<LoaderById<EmoteSet>>,
	emote_set_by_user_id_loader: DataLoader<EmoteSetByUserIdLoader>,
	global_config_loader: DataLoader<LoaderById<GlobalConfig>>,
	user_editor_by_user_id_loader: DataLoader<UserEditorByUserIdLoader>,
	user_editor_by_editor_id_loader: DataLoader<UserEditorByEditorIdLoader>,
	user_editor_by_id_loader: DataLoader<LoaderById<UserEditor>>,
	ticket_by_id_loader: DataLoader<LoaderById<Ticket>>,
	entitlement_edge_inbound_loader: DataLoader<EntitlementEdgeInboundLoader>,
	entitlement_edge_outbound_loader: DataLoader<EntitlementEdgeOutboundLoader>,
	user_by_id_loader: DataLoader<LoaderById<User>>,
	user_by_platform_id_loader: DataLoader<UserByPlatformIdLoader>,
	user_ban_by_id_loader: DataLoader<LoaderById<UserBan>>,
	user_ban_by_user_id_loader: DataLoader<UserBanByUserIdLoader>,
	user_loader: FullUserLoader,
	typesense: typesense_codegen::apis::configuration::Configuration,
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

		let event_api = EventApi::new(nats.clone(), &config.api.nats_event_subject);

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
			event_api,
			image_processor,
			audit_log_by_id_loader: LoaderById::new(db.clone()),
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
			entitlement_edge_inbound_loader: EntitlementEdgeInboundLoader::new(db.clone()),
			entitlement_edge_outbound_loader: EntitlementEdgeOutboundLoader::new(db.clone()),
			user_by_id_loader: LoaderById::new(db.clone()),
			user_by_platform_id_loader: UserByPlatformIdLoader::new(db.clone()),
			user_ban_by_id_loader: LoaderById::new(db.clone()),
			user_ban_by_user_id_loader: UserBanByUserIdLoader::new(db.clone()),
			http_client: reqwest::Client::new(),
			typesense,
			mongo,
			db,
			clickhouse,
			config,
			user_loader: FullUserLoader::new(weak.clone()),
		}))
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

	/// The ClickHouse client.
	pub fn clickhouse(&self) -> &clickhouse::Client {
		&self.clickhouse
	}

	/// The configuration.
	pub fn config(&self) -> &Config {
		&self.config
	}

	/// Global HTTP client.
	pub fn http_client(&self) -> &reqwest::Client {
		&self.http_client
	}

	/// The event API.
	pub fn event_api(&self) -> &EventApi {
		&self.event_api
	}

	/// Image processor.
	pub fn image_processor(&self) -> &ImageProcessor {
		&self.image_processor
	}

	/// The audit log loader.
	pub fn audit_log_by_id_loader(&self) -> &DataLoader<LoaderById<AuditLog>> {
		&self.audit_log_by_id_loader
	}

	/// The product loader.
	pub fn product_by_id_loader(&self) -> &DataLoader<LoaderById<Product>> {
		&self.product_by_id_loader
	}

	/// The role loader.
	pub fn role_by_id_loader(&self) -> &DataLoader<LoaderById<Role>> {
		&self.role_by_id_loader
	}

	/// The paint loader.
	pub fn paint_by_id_loader(&self) -> &DataLoader<LoaderById<Paint>> {
		&self.paint_by_id_loader
	}

	/// The badge loader.
	pub fn badge_by_id_loader(&self) -> &DataLoader<LoaderById<Badge>> {
		&self.badge_by_id_loader
	}

	/// The emote loader.
	pub fn emote_by_id_loader(&self) -> &DataLoader<LoaderById<Emote>> {
		&self.emote_by_id_loader
	}

	/// The emote by user loader.
	pub fn emote_by_user_id_loader(&self) -> &DataLoader<EmoteByUserIdLoader> {
		&self.emote_by_user_id_loader
	}

	/// The emote set loader.
	pub fn emote_set_by_id_loader(&self) -> &DataLoader<LoaderById<EmoteSet>> {
		&self.emote_set_by_id_loader
	}

	/// The emote set by user loader.
	pub fn emote_set_by_user_id_loader(&self) -> &DataLoader<EmoteSetByUserIdLoader> {
		&self.emote_set_by_user_id_loader
	}

	/// The global config loader.
	pub fn global_config_loader(&self) -> &DataLoader<LoaderById<GlobalConfig>> {
		&self.global_config_loader
	}

	/// The user editor by user loader.
	pub fn user_editor_by_user_id_loader(&self) -> &DataLoader<UserEditorByUserIdLoader> {
		&self.user_editor_by_user_id_loader
	}

	/// The user editor by editor loader.
	pub fn user_editor_by_editor_id_loader(&self) -> &DataLoader<UserEditorByEditorIdLoader> {
		&self.user_editor_by_editor_id_loader
	}

	pub fn user_editor_by_id_loader(&self) -> &DataLoader<LoaderById<UserEditor>> {
		&self.user_editor_by_id_loader
	}

	/// The ticket loader.
	pub fn ticket_by_id_loader(&self) -> &DataLoader<LoaderById<Ticket>> {
		&self.ticket_by_id_loader
	}

	/// The entitlement edge inbound loader.
	pub fn entitlement_edge_inbound_loader(&self) -> &DataLoader<EntitlementEdgeInboundLoader> {
		&self.entitlement_edge_inbound_loader
	}

	/// The entitlement edge outbound loader.
	pub fn entitlement_edge_outbound_loader(&self) -> &DataLoader<EntitlementEdgeOutboundLoader> {
		&self.entitlement_edge_outbound_loader
	}

	/// The user loader.
	pub fn user_by_id_loader(&self) -> &DataLoader<LoaderById<User>> {
		&self.user_by_id_loader
	}

	/// The user by platform ID loader.
	pub fn user_by_platform_id_loader(&self) -> &DataLoader<UserByPlatformIdLoader> {
		&self.user_by_platform_id_loader
	}

	/// The user ban loader.
	pub fn user_ban_by_id_loader(&self) -> &DataLoader<LoaderById<UserBan>> {
		&self.user_ban_by_id_loader
	}

	/// The user ban by user ID loader.
	pub fn user_ban_by_user_id_loader(&self) -> &DataLoader<UserBanByUserIdLoader> {
		&self.user_ban_by_user_id_loader
	}

	/// The full user loader.
	pub fn user_loader(&self) -> &FullUserLoader {
		&self.user_loader
	}

	/// The typesense client.
	pub fn typesense(&self) -> &typesense_codegen::apis::configuration::Configuration {
		&self.typesense
	}
}

impl HealthCheck for Global {
	fn check(&self) -> std::pin::Pin<Box<dyn futures::prelude::Future<Output = bool> + Send + '_>> {
		Box::pin(async {
			tracing::info!("running health check");

			if !match self.db().run_command(doc! { "ping": 1 }).await {
				Ok(r) => r.get_bool("ok").unwrap_or(false),
				Err(err) => {
					tracing::error!(%err, "failed to ping database");

					false
				}
			} {
				return false;
			}

			if !matches!(self.nats().connection_state(), async_nats::connection::State::Connected) {
				tracing::error!("nats not connected");
				return false;
			}

			true
		})
	}
}
