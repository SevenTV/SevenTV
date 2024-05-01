use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use chrono::Datelike;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use mongodb::bson::to_bson;
use rand::Rng;
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::runtime;
use scuffle_foundations::telementry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::{
	Collection, ProductEntitlement, ProductPurchase, ProductPurchaseStatus, User, UserEntitledCache, UserId, UserProduct,
	UserProductDataPurchase, UserProductDataSubscriptionEntryStatus,
};
use tokio::sync::{Mutex, OnceCell};
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

use crate::global::Global;

pub struct UserLoader {
	user_loader: DataLoader<InternalUserLoader>,
	user_product_purchase_loader: DataLoader<UserProductPurchaseLoader>,
	user_products_loader: DataLoader<UserProductLoader>,
	requests: Arc<Mutex<HashMap<UserId, SyncToken>>>,
}

#[derive(Clone)]
struct SyncToken {
	result: OnceCell<User>,
	done: CancellationToken,
}

struct InternalUserLoader {
	db: mongodb::Database,
}

impl InternalUserLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("InternalUserLoader", Self { db })
	}
}

impl Loader for InternalUserLoader {
	type Error = ();
	type Key = UserId;
	type Value = User;

	#[tracing::instrument(name = "InternalUserLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<Self::Value> = Self::Value::collection(&self.db)
			.find(
				mongodb::bson::doc! {
					"_id": {
						"$in": keys,
					}
				},
				None,
			)
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().map(|r| (r.id, r)).collect())
	}
}

struct UserProductPurchaseLoader {
	db: mongodb::Database,
}

impl UserProductPurchaseLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("UserProductPurchaseLoader", Self { db })
	}
}

impl Loader for UserProductPurchaseLoader {
	type Error = ();
	type Key = UserId;
	type Value = Vec<ProductPurchase>;

	#[tracing::instrument(name = "UserProductPurchaseLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Self::Value = ProductPurchase::collection(&self.db)
			.find(
				mongodb::bson::doc! {
					"$and": [
						{
							"user_id": {
								"$in": keys,
							}
						},
						{
							"status": {
								"$eq": to_bson(&ProductPurchaseStatus::Completed).unwrap(),
							}
						},
					],
				},
				None,
			)
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().into_group_map_by(|r| r.user_id.unwrap()))
	}
}

struct UserProductLoader {
	db: mongodb::Database,
}

impl UserProductLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("UserProductLoader", Self { db })
	}
}

impl Loader for UserProductLoader {
	type Error = ();
	type Key = UserId;
	type Value = Vec<UserProduct>;

	#[tracing::instrument(name = "user_product_by_user_id", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Self::Value = UserProduct::collection(&self.db)
			.find(
				mongodb::bson::doc! {
					"user_id": {
						"$in": keys,
					},
				},
				None,
			)
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().into_group_map_by(|r| r.user_id))
	}
}

impl UserLoader {
	pub fn new(db: mongodb::Database) -> Self {
		Self {
			user_loader: InternalUserLoader::new(db.clone()),
			user_product_purchase_loader: UserProductPurchaseLoader::new(db.clone()),
			user_products_loader: UserProductLoader::new(db.clone()),
			requests: Arc::new(Mutex::new(HashMap::new())),
		}
	}
}

impl UserLoader {
	pub async fn load_many(
		&self,
		global: &Arc<Global>,
		user_ids: impl IntoIterator<Item = UserId>,
	) -> Result<HashMap<UserId, User>, ()> {
		let user_ids = user_ids
			.into_iter()
			.collect::<fnv::FnvHashSet<_>>()
			.into_iter()
			.map(|id| self.load(global, id));

		let mut users = HashMap::new();

		for user in futures::future::join_all(user_ids).await {
			if let Some(user) = user? {
				users.insert(user.id, user);
			}
		}

		Ok(users)
	}

	#[tracing::instrument(name = "UserLoader::load", skip_all, fields(user_id = %user_id))]
	pub async fn load(&self, global: &Arc<Global>, user_id: UserId) -> Result<Option<User>, ()> {
		let token = {
			let mut inserted = false;

			let token = self
				.requests
				.lock()
				.await
				.entry(user_id)
				.or_insert_with(|| {
					inserted = true;
					SyncToken {
						result: OnceCell::new(),
						done: CancellationToken::new(),
					}
				})
				.clone();

			if !inserted {
				token.done.cancelled().await;
				return Ok(token.result.get().cloned());
			}

			token
		};

		runtime::spawn(Self::load_user_miss(global.clone(), token, user_id).in_current_span())
			.await
			.map_err(|err| {
				tracing::error!("failed to spawn task: {err}");
			})?
	}

	#[tracing::instrument(name = "UserLoader::load_user_miss", skip_all, fields(user_id = %user_id))]
	async fn load_user_miss(global: Arc<Global>, token: SyncToken, user_id: UserId) -> Result<Option<User>, ()> {
		tracing::Span::current().make_root();

		let _guard = token.done.drop_guard();

		let result = match global.user_by_id_loader().internal_load_fn(&global, user_id).await {
			Ok(Some(user)) => {
				token.result.set(user.clone()).ok();
				Ok(Some(user))
			}
			Ok(None) => Ok(None),
			Err(err) => {
				tracing::error!("failed to load user entitlements: {err}");
				Err(())
			}
		};

		global.user_by_id_loader().requests.lock().await.remove(&user_id);

		result
	}

	#[tracing::instrument(name = "UserLoader::internal_load_fn", skip_all, fields(user_id = %user_id))]
	async fn internal_load_fn(&self, global: &Arc<Global>, user_id: UserId) -> anyhow::Result<Option<User>> {
		let Ok(Some(mut user)) = self.user_loader.load(user_id).await else {
			return Ok(None);
		};

		if user.entitled_cache.invalidated_at > chrono::Utc::now() {
			return Ok(Some(user));
		}

		let Ok(product_purchases) = self
			.user_product_purchase_loader
			.load(user_id)
			.await
			.map(Option::unwrap_or_default)
		else {
			anyhow::bail!("failed to load user product purchases");
		};

		let product_purchases = product_purchases.into_iter().into_group_map_by(|pp| pp.product_id);

		let Ok(user_products) = self.user_products_loader.load(user_id).await.map(Option::unwrap_or_default) else {
			anyhow::bail!("failed to load user products");
		};

		let user_products = user_products
			.into_iter()
			.map(|up| (up.product_id, up))
			.collect::<HashMap<_, _>>();

		let Ok(products) = global
			.product_by_id_loader()
			.load_many(product_purchases.keys().copied())
			.await
		else {
			anyhow::bail!("failed to load products");
		};

		user.entitled_cache = UserEntitledCache {
			role_ids: user.grants.role_ids.clone(),
			badge_ids: user.grants.badge_ids.clone(),
			emote_set_ids: user.grants.emote_set_ids.clone(),
			paint_ids: user.grants.paint_ids.clone(),
			product_ids: Vec::new(),
			// 12 hours + 10%  jitter
			invalidated_at: chrono::Utc::now() + jitter(std::time::Duration::from_secs(12 * 60 * 60)),
		};

		for product in products.values() {
			let Some(purchases) = product_purchases.get(&product.id) else {
				continue;
			};

			user.entitled_cache.product_ids.push(product.id);

			product
				.entitlement_groups
				.iter()
				.filter(|group| {
					group
						.condition
						.as_ref()
						.map(|c| evaluate_expression(c, purchases, user_products.get(&product.id)))
						.unwrap_or(true)
				})
				.flat_map(|group| group.entitlements.iter().copied())
				.for_each(|entitlement| match entitlement {
					ProductEntitlement::Role(role_id) => {
						user.entitled_cache.role_ids.push(role_id);
					}
					ProductEntitlement::Badge(badge_id) => {
						user.entitled_cache.badge_ids.push(badge_id);
					}
					ProductEntitlement::Paint(paint_id) => {
						user.entitled_cache.paint_ids.push(paint_id);
					}
					ProductEntitlement::EmoteSet(emote_set_id) => {
						user.entitled_cache.emote_set_ids.push(emote_set_id);
					}
				});
		}

		let Ok(roles) = global
			.role_by_id_loader()
			.load_many(user.entitled_cache.role_ids.clone())
			.await
		else {
			anyhow::bail!("failed to load roles");
		};

		user.entitled_cache
			.badge_ids
			.extend(roles.values().flat_map(|r| r.badge_ids.iter().copied()));
		user.entitled_cache
			.paint_ids
			.extend(roles.values().flat_map(|r| r.paint_ids.iter().copied()));
		user.entitled_cache
			.emote_set_ids
			.extend(roles.values().flat_map(|r| r.emote_set_ids.iter().copied()));

		// Deduplicate
		user.entitled_cache.dedup();

		User::collection(global.db())
			.update_one(
				mongodb::bson::doc! {
					"_id": user.id,
				},
				mongodb::bson::doc! {
					"$set": {
						"entitled_cache": to_bson(&user.entitled_cache).context("failed to serialize user entitled cache")?,
					},
				},
				None,
			)
			.await
			.context("failed to update user entitled cache")?;

		Ok(Some(user))
	}
}

fn jitter(duration: std::time::Duration) -> std::time::Duration {
	let mut rng = rand::thread_rng();
	let jitter = rng.gen_range(0..duration.as_millis() / 10) as u64;
	duration + std::time::Duration::from_millis(jitter)
}

#[tracing::instrument(name = "evaluate_expression", skip(purchases, user_product))]
fn evaluate_expression(expression: &str, purchases: &[ProductPurchase], user_product: Option<&UserProduct>) -> bool {
	#[derive(serde::Serialize)]
	struct Purchase {
		date: chrono::DateTime<chrono::Utc>,
		was_gift: bool,
		price: f64,
	}

	#[derive(serde::Serialize)]
	struct UserProduct {
		created_at: chrono::DateTime<chrono::Utc>,
		duraction: Duration,
		subscription_entries: Vec<UserProductDataPurchase>,
	}

	#[derive(serde::Serialize)]
	struct Duration {
		total_years: i64,
		total_days: i64,
		total_months: i32,
	}

	let purchases = purchases
		.iter()
		.map(|pp| Purchase {
			date: pp.id.timestamp(),
			was_gift: pp.was_gift,
			price: pp.price,
		})
		.collect::<Vec<_>>();

	let user_product = user_product.map(|up| {
		let total_time = up
			.data
			.purchases
			.iter()
			.filter(|e| e.status == UserProductDataSubscriptionEntryStatus::Active)
			.map(|e| e.end - e.start)
			.sum::<chrono::Duration>();

		let created_at = up.id.timestamp();

		let packed_end_at = created_at + total_time;

		let total_months = (packed_end_at.year() - created_at.year()) * 12 + packed_end_at.month() as i32
			- created_at.month() as i32
			+ if packed_end_at.day() < created_at.day() { -1 } else { 0 };

		let duraction = Duration {
			total_days: total_time.num_days(),
			total_years: total_time.num_days() / 365,
			total_months,
		};

		UserProduct {
			created_at,
			duraction,
			subscription_entries: up.data.purchases.clone(),
		}
	});

	let result = match zen_expression::evaluate_expression(
		expression,
		&serde_json::json!({
			"purchases": purchases,
			"user_product": user_product,
		}),
	) {
		Ok(result) => result,
		Err(err) => {
			// We should consider what we want to do here.
			// This implies that the expression is invalid, so we should somehow report this
			// to the user. Rather than logging it here.
			tracing::error!(err = %err, "failed to evaluate expression");
			return false;
		}
	};

	matches!(result, serde_json::Value::Bool(true))
}
