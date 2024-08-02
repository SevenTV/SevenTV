use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;

use anyhow::Context as _;
use scuffle_foundations::batcher::dataloader::DataLoader;
use scuffle_foundations::batcher::BatcherConfig;
use scuffle_foundations::telemetry::server::HealthCheck;
use shared::database::entitlement_edge::{EntitlementEdgeInboundLoader, EntitlementEdgeOutboundLoader};

use crate::batcher::updater::MongoUpdater;
use crate::batcher::CollectionBatcher;
use crate::config::Config;
use crate::types::*;

pub struct Global {
	pub nats: async_nats::Client,
	pub jetstream: async_nats::jetstream::Context,
	pub database: mongodb::Database,
	pub config: Config,
	pub typesense: typesense_codegen::apis::configuration::Configuration,
	pub audit_log_batcher: CollectionBatcher<mongo::Event, typesense::Event>, // 1
	pub user_batcher: CollectionBatcher<mongo::User, typesense::User>,
	pub automod_rule_batcher: CollectionBatcher<mongo::AutomodRule, typesense::AutomodRule>, // 2
	pub badge_batcher: CollectionBatcher<mongo::Badge, typesense::Badge>,
	pub emote_batcher: CollectionBatcher<mongo::Emote, typesense::Emote>,
	pub emote_moderation_request_batcher:
		CollectionBatcher<mongo::EmoteModerationRequest, typesense::EmoteModerationRequest>,
	pub emote_set_batcher: CollectionBatcher<mongo::EmoteSet, typesense::EmoteSet>,
	pub page_batcher: CollectionBatcher<mongo::Page, typesense::Page>,
	pub paint_batcher: CollectionBatcher<mongo::Paint, typesense::Paint>,
	pub role_batcher: CollectionBatcher<mongo::Role, typesense::Role>,
	pub ticket_batcher: CollectionBatcher<mongo::Ticket, typesense::Ticket>,
	pub ticket_message_batcher: CollectionBatcher<mongo::TicketMessage, typesense::TicketMessage>,
	pub discount_code_batcher: CollectionBatcher<mongo::DiscountCode, typesense::DiscountCode>,
	pub gift_code_batcher: CollectionBatcher<mongo::GiftCode, typesense::GiftCode>,
	pub redeem_code_batcher: CollectionBatcher<mongo::RedeemCode, typesense::RedeemCode>,
	pub special_event_batcher: CollectionBatcher<mongo::SpecialEvent, typesense::SpecialEvent>,
	pub invoice_batcher: CollectionBatcher<mongo::Invoice, typesense::Invoice>,
	pub product_batcher: CollectionBatcher<mongo::Product, typesense::Product>,
	pub promotion_batcher: CollectionBatcher<mongo::Promotion, typesense::Promotion>,
	pub subscription_timeline_batcher: CollectionBatcher<mongo::SubscriptionTimeline, typesense::SubscriptionTimeline>,
	pub subscription_timeline_period_batcher:
		CollectionBatcher<mongo::SubscriptionTimelinePeriod, typesense::SubscriptionTimelinePeriod>,
	pub subscription_credit_batcher: CollectionBatcher<mongo::SubscriptionCredit, typesense::SubscriptionCredit>,
	pub subscription_period_batcher: CollectionBatcher<mongo::SubscriptionPeriod, typesense::SubscriptionPeriod>,
	pub user_ban_template_batcher: CollectionBatcher<mongo::UserBanTemplate, typesense::UserBanTemplate>,
	pub user_ban_batcher: CollectionBatcher<mongo::UserBan, typesense::UserBan>,
	pub user_editor_batcher: CollectionBatcher<mongo::UserEditor, typesense::UserEditor>,
	pub user_relation_batcher: CollectionBatcher<mongo::UserRelation, typesense::UserRelation>,
	pub entitlement_group_batcher: CollectionBatcher<mongo::EntitlementGroup, typesense::EntitlementGroup>, // 7
	pub entitlement_inbound_loader: DataLoader<EntitlementEdgeInboundLoader>,
	pub entitlement_outbound_loader: DataLoader<EntitlementEdgeOutboundLoader>,
	pub updater: MongoUpdater,
	is_healthy: AtomicBool,
	request_count: AtomicUsize,
	health_state: tokio::sync::Mutex<HealthCheckState>,
	semaphore: Arc<tokio::sync::Semaphore>,
}

#[derive(Debug, Default)]
struct HealthCheckState {
	nats_healthy: bool,
	db_healthy: bool,
	typesense_healthy: bool,
	last_check: Option<tokio::time::Instant>,
}

impl Global {
	pub async fn new(config: Config) -> anyhow::Result<Self> {
		let (nats, jetstream) = shared::nats::setup_nats("event-api", &config.nats)
			.await
			.context("nats connect")?;

		let database = mongodb::Client::with_uri_str(&config.database.uri)
			.await
			.context("mongo connect")?
			.default_database()
			.ok_or_else(|| anyhow::anyhow!("no default database"))?;

		let typesense = typesense_codegen::apis::configuration::Configuration {
			base_path: config.typesense.uri.clone(),
			api_key: config
				.typesense
				.api_key
				.clone()
				.map(|key| typesense_codegen::apis::configuration::ApiKey { key, prefix: None }),
			..Default::default()
		};

		Ok(Self {
			nats,
			jetstream,
			audit_log_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			user_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			automod_rule_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			badge_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			emote_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			emote_moderation_request_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			emote_set_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			page_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			paint_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			role_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			ticket_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			ticket_message_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			discount_code_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			gift_code_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			redeem_code_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			product_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			promotion_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			subscription_timeline_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			subscription_timeline_period_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			subscription_credit_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			subscription_period_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			user_ban_template_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			user_ban_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			user_editor_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			user_relation_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			special_event_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			invoice_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			entitlement_group_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			entitlement_inbound_loader: EntitlementEdgeInboundLoader::new(database.clone()),
			entitlement_outbound_loader: EntitlementEdgeOutboundLoader::new(database.clone()),
			updater: MongoUpdater::new(
				database.clone(),
				BatcherConfig {
					name: "MongoUpdater".to_string(),
					concurrency: 50,
					max_batch_size: 5_000,
					sleep_duration: std::time::Duration::from_millis(300),
				},
			),
			typesense,
			database,
			is_healthy: AtomicBool::new(false),
			request_count: AtomicUsize::new(0),
			health_state: tokio::sync::Mutex::new(HealthCheckState::default()),
			semaphore: Arc::new(tokio::sync::Semaphore::new(config.triggers.typesense_concurrency.max(1))),
			config,
		})
	}

	pub fn report_error(&self) {
		self.is_healthy.store(false, std::sync::atomic::Ordering::Relaxed);
	}

	pub fn is_healthy(&self) -> bool {
		self.is_healthy.load(std::sync::atomic::Ordering::Relaxed)
	}

	pub async fn wait_healthy(&self) -> bool {
		if self.is_healthy() {
			return true;
		}

		self.do_health_check().await
	}

	async fn do_health_check(&self) -> bool {
		let mut state = self.health_state.lock().await;
		if state
			.last_check
			.is_some_and(|t| t.elapsed() < std::time::Duration::from_secs(5))
		{
			return state.nats_healthy && state.db_healthy && state.typesense_healthy;
		}

		tracing::info!("running health check");

		state.nats_healthy = matches!(self.nats.connection_state(), async_nats::connection::State::Connected);
		if !state.nats_healthy {
			tracing::error!("nats not healthy");
		}

		state.db_healthy = match self.database.run_command(bson::doc! { "ping": 1 }).await {
			Ok(_) => true,
			Err(e) => {
				tracing::error!("mongo not healthy: {e}");
				false
			}
		};
		state.typesense_healthy = match typesense_codegen::apis::health_api::health(&self.typesense).await {
			Ok(r) => {
				if r.ok {
					true
				} else {
					tracing::error!("typesense not healthy");
					false
				}
			}
			Err(e) => {
				tracing::error!("typesense not healthy: {e}");
				false
			}
		};
		state.last_check = Some(tokio::time::Instant::now());

		self.is_healthy.store(
			state.nats_healthy && state.db_healthy && state.typesense_healthy,
			std::sync::atomic::Ordering::Relaxed,
		);

		state.nats_healthy && state.db_healthy && state.typesense_healthy
	}

	pub async fn aquire_ticket(&self) -> Option<tokio::sync::OwnedSemaphorePermit> {
		while !self.wait_healthy().await {
			tracing::warn!("waiting for mongo, typesense, and nats to be healthy");
			tokio::time::sleep(std::time::Duration::from_secs(5)).await;
		}

		self.semaphore.clone().acquire_owned().await.ok()
	}

	pub fn incr_request_count(&self) {
		self.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
	}

	pub async fn log_stats(&self) {
		let state = self.health_state.lock().await;
		let Some(last_check) = state.last_check else {
			return;
		};

		let elapsed = last_check.elapsed();

		tracing::info!(
			nats_healthy = state.nats_healthy,
			db_healthy = state.db_healthy,
			typesense_healthy = state.typesense_healthy,
			last_check = elapsed.as_secs_f64(),
			inflight = self.config.triggers.typesense_concurrency.max(1) - self.semaphore.available_permits(),
			requests = self.request_count.swap(0, std::sync::atomic::Ordering::Relaxed),
			"stats",
		);
	}
}

impl HealthCheck for Global {
	fn check(&self) -> std::pin::Pin<Box<dyn Future<Output = bool> + Send + '_>> {
		Box::pin(async { self.do_health_check().await })
	}
}