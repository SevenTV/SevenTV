#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Context as _;
use bson::doc;
use scuffle_foundations::dataloader::DataLoader;
use scuffle_foundations::telemetry::server::HealthCheck;
use shared::image_processor::ImageProcessor;

use crate::config::Config;
use crate::dataloader::audit_log::{AuditLogByActorIdLoader, AuditLogByTargetIdLoader};
use crate::dataloader::badge::BadgeByIdLoader;
use crate::dataloader::emote::{EmoteByIdLoader, EmoteByUserIdLoader};
use crate::dataloader::emote_set::{EmoteSetByIdLoader, EmoteSetByUserIdLoader};
use crate::dataloader::entitlement_edge::{EntitlementEdgeInboundLoader, EntitlementEdgeOutboundLoader};
use crate::dataloader::full_user::FullUserLoader;
use crate::dataloader::global_config::GlobalConfigLoader;
use crate::dataloader::paint::PaintByIdLoader;
use crate::dataloader::product::ProductByIdLoader;
use crate::dataloader::role::RoleByIdLoader;
use crate::dataloader::ticket::{TicketByIdLoader, TicketMessagesByTicketIdLoader};
use crate::dataloader::user::{UserByIdLoader, UserByPlatformIdLoader};
use crate::dataloader::user_editor::{UserEditorByEditorIdLoader, UserEditorByIdLoader, UserEditorByUserIdLoader};
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
	product_by_id_loader: DataLoader<ProductByIdLoader>,
	role_by_id_loader: DataLoader<RoleByIdLoader>,
	paint_by_id_loader: DataLoader<PaintByIdLoader>,
	badge_by_id_loader: DataLoader<BadgeByIdLoader>,
	emote_by_id_loader: DataLoader<EmoteByIdLoader>,
	emote_by_user_id_loader: DataLoader<EmoteByUserIdLoader>,
	emote_set_by_id_loader: DataLoader<EmoteSetByIdLoader>,
	emote_set_by_user_id_loader: DataLoader<EmoteSetByUserIdLoader>,
	global_config_loader: DataLoader<GlobalConfigLoader>,
	user_editor_by_user_id_loader: DataLoader<UserEditorByUserIdLoader>,
	user_editor_by_editor_id_loader: DataLoader<UserEditorByEditorIdLoader>,
	user_editor_by_id_loader: DataLoader<UserEditorByIdLoader>,
	ticket_by_id_loader: DataLoader<TicketByIdLoader>,
	ticket_messages_by_ticket_id_loader: DataLoader<TicketMessagesByTicketIdLoader>,
	audit_log_by_target_id_loader: DataLoader<AuditLogByTargetIdLoader>,
	audit_log_by_actor_id_loader: DataLoader<AuditLogByActorIdLoader>,
	entitlement_edge_inbound_loader: DataLoader<EntitlementEdgeInboundLoader>,
	entitlement_edge_outbound_loader: DataLoader<EntitlementEdgeOutboundLoader>,
	user_by_id_loader: DataLoader<UserByIdLoader>,
	user_by_platform_id_loader: DataLoader<UserByPlatformIdLoader>,
	user_loader: FullUserLoader,
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
		
		let event_api = EventApi::new(nats.clone());

		let image_processor = ImageProcessor::new(&config.api.image_processor)
			.await
			.context("image processor setup")?;

		Ok(Arc::new_cyclic(|weak| Self {
			nats,
			jetstream,
			event_api,
			image_processor,
			product_by_id_loader: ProductByIdLoader::new(db.clone()),
			role_by_id_loader: RoleByIdLoader::new(db.clone()),
			paint_by_id_loader: PaintByIdLoader::new(db.clone()),
			badge_by_id_loader: BadgeByIdLoader::new(db.clone()),
			emote_by_id_loader: EmoteByIdLoader::new(db.clone()),
			emote_by_user_id_loader: EmoteByUserIdLoader::new(db.clone()),
			emote_set_by_id_loader: EmoteSetByIdLoader::new(db.clone()),
			emote_set_by_user_id_loader: EmoteSetByUserIdLoader::new(db.clone()),
			global_config_loader: GlobalConfigLoader::new(db.clone()),
			user_editor_by_user_id_loader: UserEditorByUserIdLoader::new(db.clone()),
			user_editor_by_editor_id_loader: UserEditorByEditorIdLoader::new(db.clone()),
			user_editor_by_id_loader: UserEditorByIdLoader::new(db.clone()),
			ticket_by_id_loader: TicketByIdLoader::new(db.clone()),
			ticket_messages_by_ticket_id_loader: TicketMessagesByTicketIdLoader::new(db.clone()),
			audit_log_by_target_id_loader: AuditLogByTargetIdLoader::new(db.clone()),
			audit_log_by_actor_id_loader: AuditLogByActorIdLoader::new(db.clone()),
			entitlement_edge_inbound_loader: EntitlementEdgeInboundLoader::new(db.clone()),
			entitlement_edge_outbound_loader: EntitlementEdgeOutboundLoader::new(db.clone()),
			user_by_id_loader: UserByIdLoader::new(db.clone()),
			user_by_platform_id_loader: UserByPlatformIdLoader::new(db.clone()),
			http_client: reqwest::Client::new(),
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

	/// The product loader.
	pub fn product_by_id_loader(&self) -> &DataLoader<ProductByIdLoader> {
		&self.product_by_id_loader
	}

	/// The role loader.
	pub fn role_by_id_loader(&self) -> &DataLoader<RoleByIdLoader> {
		&self.role_by_id_loader
	}

	/// The paint loader.
	pub fn paint_by_id_loader(&self) -> &DataLoader<PaintByIdLoader> {
		&self.paint_by_id_loader
	}

	/// The badge loader.
	pub fn badge_by_id_loader(&self) -> &DataLoader<BadgeByIdLoader> {
		&self.badge_by_id_loader
	}

	/// The emote loader.
	pub fn emote_by_id_loader(&self) -> &DataLoader<EmoteByIdLoader> {
		&self.emote_by_id_loader
	}

	/// The emote by user loader.
	pub fn emote_by_user_id_loader(&self) -> &DataLoader<EmoteByUserIdLoader> {
		&self.emote_by_user_id_loader
	}

	/// The emote set loader.
	pub fn emote_set_by_id_loader(&self) -> &DataLoader<EmoteSetByIdLoader> {
		&self.emote_set_by_id_loader
	}

	/// The emote set by user loader.
	pub fn emote_set_by_user_id_loader(&self) -> &DataLoader<EmoteSetByUserIdLoader> {
		&self.emote_set_by_user_id_loader
	}

	/// The global config loader.
	pub fn global_config_loader(&self) -> &DataLoader<GlobalConfigLoader> {
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

	pub fn user_editor_by_id_loader(&self) -> &DataLoader<UserEditorByIdLoader> {
		&self.user_editor_by_id_loader
	}

	/// The ticket loader.
	pub fn ticket_by_id_loader(&self) -> &DataLoader<TicketByIdLoader> {
		&self.ticket_by_id_loader
	}

	/// The ticket messages loader.
	pub fn ticket_messages_by_ticket_id_loader(&self) -> &DataLoader<TicketMessagesByTicketIdLoader> {
		&self.ticket_messages_by_ticket_id_loader
	}

	/// The audit log by target id loader.
	pub fn audit_log_by_target_id_loader(&self) -> &DataLoader<AuditLogByTargetIdLoader> {
		&self.audit_log_by_target_id_loader
	}

	/// The audit log by actor loader.
	pub fn audit_log_by_actor_id_loader(&self) -> &DataLoader<AuditLogByActorIdLoader> {
		&self.audit_log_by_actor_id_loader
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
	pub fn user_by_id_loader(&self) -> &DataLoader<UserByIdLoader> {
		&self.user_by_id_loader
	}

	/// The user by platform ID loader.
	pub fn user_by_platform_id_loader(&self) -> &DataLoader<UserByPlatformIdLoader> {
		&self.user_by_platform_id_loader
	}

	/// The full user loader.
	pub fn user_loader(&self) -> &FullUserLoader {
		&self.user_loader
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
