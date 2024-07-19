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
	nats: async_nats::Client,
	jetstream: async_nats::jetstream::Context,
	database: mongodb::Database,
	config: Config,
	typesense: typesense_codegen::apis::configuration::Configuration,
	audit_log_batcher: CollectionBatcher<mongo::AuditLog, typesense::AuditLog>, // 1
	user_batcher: CollectionBatcher<mongo::User, typesense::User>,
	automod_rule_batcher: CollectionBatcher<mongo::AutomodRule, typesense::AutomodRule>, // 2
	badge_batcher: CollectionBatcher<mongo::Badge, typesense::Badge>,
	emote_batcher: CollectionBatcher<mongo::Emote, typesense::Emote>,
	emote_moderation_request_batcher: CollectionBatcher<mongo::EmoteModerationRequest, typesense::EmoteModerationRequest>,
	emote_set_batcher: CollectionBatcher<mongo::EmoteSet, typesense::EmoteSet>,
	page_batcher: CollectionBatcher<mongo::Page, typesense::Page>,
	paint_batcher: CollectionBatcher<mongo::Paint, typesense::Paint>,
	role_batcher: CollectionBatcher<mongo::Role, typesense::Role>,
	ticket_batcher: CollectionBatcher<mongo::Ticket, typesense::Ticket>,
	ticket_message_batcher: CollectionBatcher<mongo::TicketMessage, typesense::TicketMessage>,
	discount_code_batcher: CollectionBatcher<mongo::DiscountCode, typesense::DiscountCode>,
	gift_code_batcher: CollectionBatcher<mongo::GiftCode, typesense::GiftCode>,
	redeem_code_batcher: CollectionBatcher<mongo::RedeemCode, typesense::RedeemCode>,
	special_event_batcher: CollectionBatcher<mongo::SpecialEvent, typesense::SpecialEvent>,
	invoice_batcher: CollectionBatcher<mongo::Invoice, typesense::Invoice>,
	product_batcher: CollectionBatcher<mongo::Product, typesense::Product>,
	promotion_batcher: CollectionBatcher<mongo::Promotion, typesense::Promotion>,
	subscription_timeline_batcher: CollectionBatcher<mongo::SubscriptionTimeline, typesense::SubscriptionTimeline>,
	subscription_timeline_period_batcher:
		CollectionBatcher<mongo::SubscriptionTimelinePeriod, typesense::SubscriptionTimelinePeriod>,
	subscription_credit_batcher: CollectionBatcher<mongo::SubscriptionCredit, typesense::SubscriptionCredit>,
	subscription_period_batcher: CollectionBatcher<mongo::SubscriptionPeriod, typesense::SubscriptionPeriod>,
	user_ban_template_batcher: CollectionBatcher<mongo::UserBanTemplate, typesense::UserBanTemplate>,
	user_ban_batcher: CollectionBatcher<mongo::UserBan, typesense::UserBan>,
	user_editor_batcher: CollectionBatcher<mongo::UserEditor, typesense::UserEditor>,
	user_relation_batcher: CollectionBatcher<mongo::UserRelation, typesense::UserRelation>,
	entitlement_group_batcher: CollectionBatcher<mongo::EntitlementGroup, typesense::EntitlementGroup>, // 7
	entitlement_inbound_loader: DataLoader<EntitlementEdgeInboundLoader>,
	entitlement_outbound_loader: DataLoader<EntitlementEdgeOutboundLoader>,
	updater: MongoUpdater,
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

	pub fn nats(&self) -> &async_nats::Client {
		&self.nats
	}

	pub fn jetstream(&self) -> &async_nats::jetstream::Context {
		&self.jetstream
	}

	pub fn db(&self) -> &mongodb::Database {
		&self.database
	}

	pub fn config(&self) -> &Config {
		&self.config
	}

	pub fn typesense(&self) -> &typesense_codegen::apis::configuration::Configuration {
		&self.typesense
	}

	pub fn audit_log_batcher(&self) -> &CollectionBatcher<mongo::AuditLog, typesense::AuditLog> {
		&self.audit_log_batcher
	}

	pub fn user_batcher(&self) -> &CollectionBatcher<mongo::User, typesense::User> {
		&self.user_batcher
	}

	pub fn automod_rule_batcher(&self) -> &CollectionBatcher<mongo::AutomodRule, typesense::AutomodRule> {
		&self.automod_rule_batcher
	}

	pub fn badge_batcher(&self) -> &CollectionBatcher<mongo::Badge, typesense::Badge> {
		&self.badge_batcher
	}

	pub fn emote_batcher(&self) -> &CollectionBatcher<mongo::Emote, typesense::Emote> {
		&self.emote_batcher
	}

	pub fn emote_moderation_request_batcher(
		&self,
	) -> &CollectionBatcher<mongo::EmoteModerationRequest, typesense::EmoteModerationRequest> {
		&self.emote_moderation_request_batcher
	}

	pub fn emote_set_batcher(&self) -> &CollectionBatcher<mongo::EmoteSet, typesense::EmoteSet> {
		&self.emote_set_batcher
	}

	pub fn page_batcher(&self) -> &CollectionBatcher<mongo::Page, typesense::Page> {
		&self.page_batcher
	}

	pub fn paint_batcher(&self) -> &CollectionBatcher<mongo::Paint, typesense::Paint> {
		&self.paint_batcher
	}

	pub fn role_batcher(&self) -> &CollectionBatcher<mongo::Role, typesense::Role> {
		&self.role_batcher
	}

	pub fn ticket_batcher(&self) -> &CollectionBatcher<mongo::Ticket, typesense::Ticket> {
		&self.ticket_batcher
	}

	pub fn ticket_message_batcher(&self) -> &CollectionBatcher<mongo::TicketMessage, typesense::TicketMessage> {
		&self.ticket_message_batcher
	}

	pub fn discount_code_batcher(&self) -> &CollectionBatcher<mongo::DiscountCode, typesense::DiscountCode> {
		&self.discount_code_batcher
	}

	pub fn gift_code_batcher(&self) -> &CollectionBatcher<mongo::GiftCode, typesense::GiftCode> {
		&self.gift_code_batcher
	}

	pub fn redeem_code_batcher(&self) -> &CollectionBatcher<mongo::RedeemCode, typesense::RedeemCode> {
		&self.redeem_code_batcher
	}

	pub fn special_event_batcher(&self) -> &CollectionBatcher<mongo::SpecialEvent, typesense::SpecialEvent> {
		&self.special_event_batcher
	}

	pub fn invoice_batcher(&self) -> &CollectionBatcher<mongo::Invoice, typesense::Invoice> {
		&self.invoice_batcher
	}

	pub fn product_batcher(&self) -> &CollectionBatcher<mongo::Product, typesense::Product> {
		&self.product_batcher
	}

	pub fn promotion_batcher(&self) -> &CollectionBatcher<mongo::Promotion, typesense::Promotion> {
		&self.promotion_batcher
	}

	pub fn subscription_timeline_batcher(
		&self,
	) -> &CollectionBatcher<mongo::SubscriptionTimeline, typesense::SubscriptionTimeline> {
		&self.subscription_timeline_batcher
	}

	pub fn subscription_timeline_period_batcher(
		&self,
	) -> &CollectionBatcher<mongo::SubscriptionTimelinePeriod, typesense::SubscriptionTimelinePeriod> {
		&self.subscription_timeline_period_batcher
	}

	pub fn subscription_credit_batcher(
		&self,
	) -> &CollectionBatcher<mongo::SubscriptionCredit, typesense::SubscriptionCredit> {
		&self.subscription_credit_batcher
	}

	pub fn subscription_period_batcher(
		&self,
	) -> &CollectionBatcher<mongo::SubscriptionPeriod, typesense::SubscriptionPeriod> {
		&self.subscription_period_batcher
	}

	pub fn user_ban_template_batcher(&self) -> &CollectionBatcher<mongo::UserBanTemplate, typesense::UserBanTemplate> {
		&self.user_ban_template_batcher
	}

	pub fn user_ban_batcher(&self) -> &CollectionBatcher<mongo::UserBan, typesense::UserBan> {
		&self.user_ban_batcher
	}

	pub fn user_editor_batcher(&self) -> &CollectionBatcher<mongo::UserEditor, typesense::UserEditor> {
		&self.user_editor_batcher
	}

	pub fn user_relation_batcher(&self) -> &CollectionBatcher<mongo::UserRelation, typesense::UserRelation> {
		&self.user_relation_batcher
	}

	pub fn entitlement_group_batcher(&self) -> &CollectionBatcher<mongo::EntitlementGroup, typesense::EntitlementGroup> {
		&self.entitlement_group_batcher
	}

	pub fn entitlement_inbound_loader(&self) -> &DataLoader<EntitlementEdgeInboundLoader> {
		&self.entitlement_inbound_loader
	}

	pub fn entitlement_outbound_loader(&self) -> &DataLoader<EntitlementEdgeOutboundLoader> {
		&self.entitlement_outbound_loader
	}

	pub fn updater(&self) -> &MongoUpdater {
		&self.updater
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

		state.nats_healthy = matches!(self.nats().connection_state(), async_nats::connection::State::Connected);
		if !state.nats_healthy {
			tracing::error!("nats not healthy");
		}

		state.db_healthy = match self.db().run_command(bson::doc! { "ping": 1 }).await {
			Ok(_) => true,
			Err(e) => {
				tracing::error!("mongo not healthy: {e}");
				false
			}
		};
		state.typesense_healthy = match typesense_codegen::apis::health_api::health(self.typesense()).await {
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
