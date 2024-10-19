use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;

use anyhow::Context as _;
use scuffle_foundations::batcher::dataloader::DataLoader;
use scuffle_foundations::batcher::{Batcher, BatcherConfig};
use scuffle_foundations::telemetry::server::HealthCheck;
use shared::clickhouse::emote_stat::EmoteStat;
use shared::database::entitlement_edge::{EntitlementEdgeInboundLoader, EntitlementEdgeOutboundLoader};
use shared::database::updater::MongoUpdater;
use typesense_rs::apis::Api;

use crate::batcher::clickhouse::ClickhouseInsert;
use crate::batcher::CollectionBatcher;
use crate::config::Config;
use crate::types::*;

pub struct Global {
	pub nats: async_nats::Client,
	pub jetstream: async_nats::jetstream::Context,
	pub database: mongodb::Database,
	pub config: Config,
	pub typesense: Arc<typesense_rs::apis::ApiClient>,
	pub event_batcher: CollectionBatcher<mongo::StoredEvent>,
	pub user_batcher: CollectionBatcher<mongo::User>,
	pub automod_rule_batcher: CollectionBatcher<mongo::AutomodRule>,
	pub badge_batcher: CollectionBatcher<mongo::Badge>,
	pub emote_batcher: CollectionBatcher<mongo::Emote>,
	pub emote_moderation_request_batcher: CollectionBatcher<mongo::EmoteModerationRequest>,
	pub emote_set_batcher: CollectionBatcher<mongo::EmoteSet>,
	pub page_batcher: CollectionBatcher<mongo::Page>,
	pub paint_batcher: CollectionBatcher<mongo::Paint>,
	pub role_batcher: CollectionBatcher<mongo::Role>,
	pub ticket_batcher: CollectionBatcher<mongo::Ticket>,
	pub ticket_message_batcher: CollectionBatcher<mongo::TicketMessage>,
	pub redeem_code_batcher: CollectionBatcher<mongo::RedeemCode>,
	pub special_event_batcher: CollectionBatcher<mongo::SpecialEvent>,
	pub invoice_batcher: CollectionBatcher<mongo::Invoice>,
	pub product_batcher: CollectionBatcher<mongo::Product>,
	pub subscription_period_batcher: CollectionBatcher<mongo::SubscriptionPeriod>,
	pub user_ban_template_batcher: CollectionBatcher<mongo::UserBanTemplate>,
	pub user_ban_batcher: CollectionBatcher<mongo::UserBan>,
	pub user_editor_batcher: CollectionBatcher<mongo::UserEditor>,
	pub user_relation_batcher: CollectionBatcher<mongo::UserRelation>,
	pub entitlement_inbound_loader: DataLoader<EntitlementEdgeInboundLoader>,
	pub entitlement_outbound_loader: DataLoader<EntitlementEdgeOutboundLoader>,
	pub emote_stats_batcher: Batcher<ClickhouseInsert<EmoteStat>>,
	pub subscription_product_batcher: CollectionBatcher<mongo::SubscriptionProduct>,
	pub subscription_batcher: CollectionBatcher<mongo::Subscription>,
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

		let typesense = Arc::new(typesense_rs::apis::ApiClient::new(Arc::new(
			typesense_rs::apis::configuration::Configuration {
				base_path: config.typesense.uri.clone(),
				api_key: config
					.typesense
					.api_key
					.clone()
					.map(|key| typesense_rs::apis::configuration::ApiKey { key, prefix: None }),
				..Default::default()
			},
		)));

		let clickhouse = shared::clickhouse::init_clickhouse(&config.clickhouse).await?;

		Ok(Self {
			nats,
			jetstream,
			event_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
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
			redeem_code_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			product_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			subscription_period_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			user_ban_template_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			user_ban_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			user_editor_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			user_relation_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			special_event_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			invoice_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			entitlement_inbound_loader: EntitlementEdgeInboundLoader::new(database.clone()),
			entitlement_outbound_loader: EntitlementEdgeOutboundLoader::new(database.clone()),
			subscription_product_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
			subscription_batcher: CollectionBatcher::new(database.clone(), typesense.clone()),
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
			emote_stats_batcher: ClickhouseInsert::new(clickhouse),
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

		tracing::debug!("running health check");

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
		state.typesense_healthy = match self.typesense.health_api().health().await {
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

	pub fn request_count(&self) -> usize {
		self.request_count.load(std::sync::atomic::Ordering::Relaxed)
	}

	pub async fn reindex(&self) {
		macro_rules! reindex_collection {
			($($collection:ty),*$(,)?) => {
				{
					[
						$(
							shared::database::updater::MongoReq::update::<$collection>(
								shared::database::queries::filter::filter! {
									$collection {
										search_updated_at: &None,
									}
								},
								shared::database::queries::update::update! {
									#[query(set)]
									$collection {
										updated_at: chrono::Utc::now(),
									}
								},
								true,
							),
						)*
					]
				}
			}
		}

		for result in self
			.updater
			.bulk(reindex_collection! {
				crate::types::mongo::RedeemCode,
				crate::types::mongo::SpecialEvent,
				crate::types::mongo::Invoice,
				crate::types::mongo::Product,
				crate::types::mongo::SubscriptionProduct,
				crate::types::mongo::SubscriptionPeriod,
				crate::types::mongo::UserBanTemplate,
				crate::types::mongo::UserBan,
				crate::types::mongo::UserEditor,
				crate::types::mongo::User,
				crate::types::mongo::UserRelation,
				crate::types::mongo::StoredEvent,
				crate::types::mongo::AutomodRule,
				crate::types::mongo::Badge,
				crate::types::mongo::EmoteModerationRequest,
				crate::types::mongo::EmoteSet,
				crate::types::mongo::Emote,
				crate::types::mongo::Page,
				crate::types::mongo::Paint,
				crate::types::mongo::Role,
				crate::types::mongo::Ticket,
				crate::types::mongo::TicketMessage,
				crate::types::mongo::Subscription,
			})
			.await
		{
			if let Err(e) = result {
				tracing::error!("failed to reindex: {e}");
			}
		}
	}
}

impl HealthCheck for Global {
	fn check(&self) -> std::pin::Pin<Box<dyn Future<Output = bool> + Send + '_>> {
		Box::pin(async { self.do_health_check().await })
	}
}
