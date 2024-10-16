#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Context as _;
use bson::doc;
use scuffle_foundations::batcher::dataloader::DataLoader;
use scuffle_foundations::batcher::BatcherConfig;
use scuffle_foundations::telemetry::server::HealthCheck;
use shared::clickhouse::init_clickhouse;
use shared::database::badge::Badge;
use shared::database::emote_moderation_request::EmoteModerationRequest;
use shared::database::emote_set::EmoteSet;
use shared::database::entitlement_edge::{EntitlementEdgeInboundLoader, EntitlementEdgeOutboundLoader};
use shared::database::global::GlobalConfig;
use shared::database::loader::LoaderById;
use shared::database::paint::Paint;
use shared::database::product::codes::RedeemCode;
use shared::database::product::subscription::Subscription;
use shared::database::product::{Product, SubscriptionProduct};
use shared::database::role::Role;
use shared::database::stored_event::StoredEvent;
use shared::database::ticket::Ticket;
use shared::database::updater::MongoUpdater;
use shared::database::user::ban::UserBan;
use shared::database::user::editor::UserEditor;
use shared::database::user::profile_picture::UserProfilePicture;
use shared::database::user::session::UserSession;
use shared::database::user::User;
use shared::image_processor::ImageProcessor;
use shared::ip::GeoIpResolver;
use shared::redis::setup_redis;

use crate::config::Config;
use crate::dataloader::active_subscription_period::{
	ActiveSubscriptionPeriodByUserIdLoader, SubscriptionPeriodsByUserIdLoader,
};
use crate::dataloader::emote::{EmoteByIdLoader, EmoteByUserIdLoader};
use crate::dataloader::emote_set::EmoteSetByUserIdLoader;
use crate::dataloader::full_user::FullUserLoader;
use crate::dataloader::subscription_products::SubscriptionProductsLoader;
use crate::dataloader::ticket_message::TicketMessageByTicketIdLoader;
use crate::dataloader::user::UserByPlatformIdLoader;
use crate::dataloader::user_ban::UserBanByUserIdLoader;
use crate::dataloader::user_editor::{UserEditorByEditorIdLoader, UserEditorByUserIdLoader};
use crate::dataloader::user_session::UserSessionUpdaterBatcher;
use crate::ratelimit::RateLimiter;
use crate::stripe_client;

pub struct Global {
	pub nats: async_nats::Client,
	pub redis: fred::clients::RedisClient,
	pub rate_limiter: RateLimiter,
	geoip: Option<GeoIpResolver>,
	pub jetstream: async_nats::jetstream::Context,
	pub config: Config,
	pub mongo: mongodb::Client,
	pub db: mongodb::Database,
	pub clickhouse: clickhouse::Client,
	pub http_client: reqwest::Client,
	pub stripe_client: stripe_client::StripeClientManager,
	pub image_processor: ImageProcessor,
	pub event_by_id_loader: DataLoader<LoaderById<StoredEvent>>,
	pub product_by_id_loader: DataLoader<LoaderById<Product>>,
	pub role_by_id_loader: DataLoader<LoaderById<Role>>,
	pub paint_by_id_loader: DataLoader<LoaderById<Paint>>,
	pub badge_by_id_loader: DataLoader<LoaderById<Badge>>,
	pub emote_by_id_loader: DataLoader<EmoteByIdLoader>,
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
	pub subscription_product_by_id_loader: DataLoader<LoaderById<SubscriptionProduct>>,
	pub subscription_products_loader: DataLoader<SubscriptionProductsLoader>,
	pub subscription_by_id_loader: DataLoader<LoaderById<Subscription>>,
	pub subscription_periods_by_user_id_loader: DataLoader<SubscriptionPeriodsByUserIdLoader>,
	pub active_subscription_period_by_user_id_loader: DataLoader<ActiveSubscriptionPeriodByUserIdLoader>,
	pub redeem_code_by_id_loader: DataLoader<LoaderById<RedeemCode>>,
	pub user_by_id_loader: DataLoader<LoaderById<User>>,
	pub user_by_platform_id_loader: DataLoader<UserByPlatformIdLoader>,
	pub user_ban_by_id_loader: DataLoader<LoaderById<UserBan>>,
	pub user_ban_by_user_id_loader: DataLoader<UserBanByUserIdLoader>,
	pub user_profile_picture_id_loader: DataLoader<LoaderById<UserProfilePicture>>,
	pub emote_moderation_request_by_id_loader: DataLoader<LoaderById<EmoteModerationRequest>>,
	pub user_session_by_id_loader: DataLoader<LoaderById<UserSession>>,
	pub user_session_updater_batcher: DataLoader<UserSessionUpdaterBatcher>,
	pub user_loader: FullUserLoader,
	pub typesense: typesense_codegen::apis::configuration::Configuration,
	pub updater: MongoUpdater,
}

impl Global {
	pub async fn new(config: Config) -> anyhow::Result<Arc<Self>> {
		let (nats, jetstream) = shared::nats::setup_nats("api", &config.nats).await.context("nats connect")?;

		tracing::info!("connected to nats");

		let mongo = shared::database::setup_and_init_database(&config.database)
			.await
			.context("database setup")?;

		tracing::info!("connected to mongo");

		let db = mongo
			.default_database()
			.ok_or_else(|| anyhow::anyhow!("No default database"))?;

		let clickhouse = init_clickhouse(&config.clickhouse).await?;

		let image_processor = ImageProcessor::new(&config.image_processor)
			.await
			.context("image processor setup")?;

		tracing::info!("connected to image processor");

		let typesense = typesense_codegen::apis::configuration::Configuration {
			base_path: config.typesense.uri.clone(),
			api_key: config
				.typesense
				.api_key
				.clone()
				.map(|key| typesense_codegen::apis::configuration::ApiKey { key, prefix: None }),
			..Default::default()
		};

		let stripe_client = stripe_client::StripeClientManager::new(&config);

		let geoip = if let Some(config) = config.geoip.as_ref() {
			Some(GeoIpResolver::new(config).await.context("geoip resolver")?)
		} else {
			None
		};

		let redis = setup_redis(&config.redis).await?;

		tracing::info!("connected to redis");

		let rate_limiter = RateLimiter::new(redis.clone()).await?;

		tracing::info!("connected to rate limiter");

		Ok(Arc::new_cyclic(|weak| Self {
			nats,
			geoip,
			redis,
			rate_limiter,
			jetstream,
			image_processor,
			event_by_id_loader: LoaderById::new(db.clone()),
			product_by_id_loader: LoaderById::new(db.clone()),
			role_by_id_loader: LoaderById::new(db.clone()),
			paint_by_id_loader: LoaderById::new(db.clone()),
			badge_by_id_loader: LoaderById::new(db.clone()),
			emote_by_id_loader: EmoteByIdLoader::new(db.clone()),
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
			subscription_product_by_id_loader: LoaderById::new(db.clone()),
			subscription_products_loader: SubscriptionProductsLoader::new(db.clone()),
			subscription_by_id_loader: LoaderById::new(db.clone()),
			subscription_periods_by_user_id_loader: SubscriptionPeriodsByUserIdLoader::new(db.clone()),
			active_subscription_period_by_user_id_loader: ActiveSubscriptionPeriodByUserIdLoader::new(db.clone()),
			redeem_code_by_id_loader: LoaderById::new(db.clone()),
			user_by_id_loader: LoaderById::new(db.clone()),
			user_by_platform_id_loader: UserByPlatformIdLoader::new(db.clone()),
			user_ban_by_id_loader: LoaderById::new(db.clone()),
			user_ban_by_user_id_loader: UserBanByUserIdLoader::new(db.clone()),
			user_profile_picture_id_loader: LoaderById::new(db.clone()),
			emote_moderation_request_by_id_loader: LoaderById::new(db.clone()),
			user_session_by_id_loader: LoaderById::new(db.clone()),
			user_session_updater_batcher: UserSessionUpdaterBatcher::new(db.clone()),
			http_client: reqwest::Client::new(),
			stripe_client,
			typesense,
			mongo,
			updater: MongoUpdater::new(
				db.clone(),
				BatcherConfig {
					name: "MongoUpdater".to_string(),
					concurrency: 50,
					max_batch_size: 5_000,
					sleep_duration: std::time::Duration::from_millis(300),
				},
			),
			db,
			clickhouse,
			config,
			user_loader: FullUserLoader::new(weak.clone()),
		}))
	}

	pub fn geoip(&self) -> Option<&GeoIpResolver> {
		self.geoip.as_ref()
	}
}

impl HealthCheck for Global {
	fn check(&self) -> std::pin::Pin<Box<dyn futures::prelude::Future<Output = bool> + Send + '_>> {
		Box::pin(async {
			tracing::debug!("running health check");

			if let Err(err) = self.db.run_command(doc! { "ping": 1 }).await {
				tracing::error!(%err, "failed to ping database");
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
