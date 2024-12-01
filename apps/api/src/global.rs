#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Context as _;
use scuffle_batching::DataLoader;
use scuffle_bootstrap_telemetry::opentelemetry;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::metrics::SdkMeterProvider;
use scuffle_bootstrap_telemetry::opentelemetry_sdk::Resource;
use scuffle_metrics::opentelemetry::KeyValue;
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
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

use crate::config::Config;
use crate::dataloader::active_subscription_period::{
	ActiveSubscriptionPeriodByUserIdLoader, SubscriptionPeriodsByUserIdLoader,
};
use crate::dataloader::emote::{EmoteByIdLoader, EmoteByUserIdLoader};
use crate::dataloader::emote_set::EmoteSetByUserIdLoader;
use crate::dataloader::full_user::FullUserLoader;
use crate::dataloader::subscription_products::SubscriptionProductsLoader;
use crate::dataloader::ticket_message::TicketMessageByTicketIdLoader;
use crate::dataloader::user::{UserByPlatformIdLoader, UserByPlatformUsernameLoader};
use crate::dataloader::user_ban::UserBanByUserIdLoader;
use crate::dataloader::user_editor::{UserEditorByEditorIdLoader, UserEditorByUserIdLoader};
use crate::dataloader::user_session::UserSessionUpdaterBatcher;
use crate::http::v4;
use crate::mutex::DistributedMutex;
use crate::ratelimit::RateLimiter;
use crate::stripe_client;

pub struct Global {
	pub nats: async_nats::Client,
	pub redis: fred::clients::Pool,
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
	pub user_by_platform_username_loader: DataLoader<UserByPlatformUsernameLoader>,
	pub user_ban_by_id_loader: DataLoader<LoaderById<UserBan>>,
	pub user_ban_by_user_id_loader: DataLoader<UserBanByUserIdLoader>,
	pub user_profile_picture_id_loader: DataLoader<LoaderById<UserProfilePicture>>,
	pub emote_moderation_request_by_id_loader: DataLoader<LoaderById<EmoteModerationRequest>>,
	pub user_session_by_id_loader: DataLoader<LoaderById<UserSession>>,
	pub user_session_updater_batcher: DataLoader<UserSessionUpdaterBatcher>,
	pub user_loader: FullUserLoader,
	pub typesense: typesense_rs::apis::ApiClient,
	pub updater: MongoUpdater,
	pub mutex: DistributedMutex,
	metrics_registry: scuffle_bootstrap_telemetry::prometheus::Registry,
}

impl scuffle_bootstrap::global::Global for Global {
	type Config = Config;

	fn pre_init() -> anyhow::Result<()> {
		rustls::crypto::aws_lc_rs::default_provider().install_default().ok();
		Ok(())
	}

	async fn init(config: Config) -> anyhow::Result<Arc<Self>> {
		let metrics_registry = scuffle_bootstrap_telemetry::prometheus::Registry::new();

		opentelemetry::global::set_meter_provider(
			SdkMeterProvider::builder()
				.with_resource(Resource::new(vec![KeyValue::new("service.name", env!("CARGO_BIN_NAME"))]))
				.with_reader(
					scuffle_metrics::prometheus::exporter()
						.with_registry(metrics_registry.clone())
						.build()
						.context("prometheus metrics exporter")?,
				)
				.build(),
		);

		tracing_subscriber::registry()
			.with(
				tracing_subscriber::fmt::layer()
					.with_file(true)
					.with_line_number(true)
					.with_filter(
						EnvFilter::builder()
							.with_default_directive(LevelFilter::INFO.into())
							.parse_lossy(&config.level),
					),
			)
			.init();

		tracing::info!("starting api");

		if let Some(path) = config.export_schema_path {
			tracing::info!("exporting graphql schema to {}", path.display());
			std::fs::write(path, v4::export_gql_schema()).context("exporting graphql schema")?;
			std::process::exit(0);
		}

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

		let typesense = typesense_rs::apis::configuration::Configuration {
			base_path: config.typesense.uri.clone(),
			api_key: config
				.typesense
				.api_key
				.clone()
				.map(|key| typesense_rs::apis::configuration::ApiKey { key, prefix: None }),
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

		let rate_limiter = RateLimiter::new(redis.clone()).await.context("rate limiter")?;

		tracing::info!("connected to redis rate limiter");

		let mutex = DistributedMutex::new(redis.clone()).await.context("mutex")?;

		tracing::info!("connected to redis mutex");

		Ok(Arc::new_cyclic(|weak| Self {
			nats,
			geoip,
			redis,
			rate_limiter,
			mutex,
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
			user_by_platform_username_loader: UserByPlatformUsernameLoader::new(db.clone()),
			user_ban_by_id_loader: LoaderById::new(db.clone()),
			user_ban_by_user_id_loader: UserBanByUserIdLoader::new(db.clone()),
			user_profile_picture_id_loader: LoaderById::new(db.clone()),
			emote_moderation_request_by_id_loader: LoaderById::new(db.clone()),
			user_session_by_id_loader: LoaderById::new(db.clone()),
			user_session_updater_batcher: UserSessionUpdaterBatcher::new(db.clone()),
			http_client: reqwest::Client::new(),
			stripe_client,
			typesense: typesense_rs::apis::ApiClient::new(Arc::new(typesense)),
			mongo,
			updater: MongoUpdater::new(db.clone(), 500, 50, std::time::Duration::from_millis(300)),
			db,
			clickhouse,
			config,
			metrics_registry,
			user_loader: FullUserLoader::new(weak.clone()),
		}))
	}

	async fn on_services_start(self: &Arc<Self>) -> anyhow::Result<()> {
		tracing::info!("api running");
		Ok(())
	}

	async fn on_service_exit(
		self: &Arc<Self>,
		name: &'static str,
		result: anyhow::Result<()>,
	) -> anyhow::Result<()> {
		if let Err(err) = &result {
			tracing::error!("service {name} exited with error: {:#}", err);
		} else {
			tracing::info!("service {name} exited");
		}

		result
	}

	async fn on_exit(self: &Arc<Self>, result: anyhow::Result<()>) -> anyhow::Result<()> {
		tracing::info!("api exiting");
		result
	}
}

impl Global {
	pub fn geoip(&self) -> Option<&GeoIpResolver> {
		self.geoip.as_ref()
	}
}

impl scuffle_signal::SignalConfig for Global {
	async fn on_shutdown(self: &std::sync::Arc<Self>) -> anyhow::Result<()> {
		tracing::info!("shutting down");
		Ok(())
	}
}

impl scuffle_bootstrap_telemetry::TelemetryConfig for Global {
	async fn health_check(&self) -> Result<(), anyhow::Error> {
		tracing::debug!("running health check");

		if let Err(err) = self.db.run_command(bson::doc! { "ping": 1 }).await {
			anyhow::bail!("failed to ping database: {err}");
		}

		if !matches!(self.nats.connection_state(), async_nats::connection::State::Connected) {
			anyhow::bail!("nats not connected");
		}

		Ok(())
	}

	fn bind_address(&self) -> Option<std::net::SocketAddr> {
		self.config.metrics_bind_address
	}

	fn prometheus_metrics_registry(&self) -> Option<&scuffle_bootstrap_telemetry::prometheus::Registry> {
		Some(&self.metrics_registry)
	}
}
