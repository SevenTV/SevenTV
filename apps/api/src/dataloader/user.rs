use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use chrono::{Datelike, TimeZone};
use rand::Rng;
use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use tokio::sync::{Mutex, OnceCell};
use tokio_util::sync::CancellationToken;
use ulid::Ulid;

use crate::database::{
	ProductEntitlement, ProductPurchase, ProductPurchaseStatus, User, UserEntitledCache, UserProduct,
	UserProductDataPurchase, UserProductDataSubscriptionEntryStatus,
};
use crate::global::Global;

pub struct UserLoader {
	user_role_loader: DataLoader<UserRoleLoader>,
	user_badge_loader: DataLoader<UserBadgeLoader>,
	user_paint_loader: DataLoader<UserPaintLoader>,
	user_emote_set_loader: DataLoader<UserEmoteSetLoader>,
	user_loader: DataLoader<InternalUserLoader>,
	user_product_purchase_loader: DataLoader<UserProductPurchaseLoader>,
	user_products_loader: DataLoader<UserProductLoader>,
	requests: Mutex<HashMap<Ulid, SyncToken>>,
}

#[derive(Clone)]
struct SyncToken {
	result: OnceCell<User>,
	done: CancellationToken,
}

struct UserRoleLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl UserRoleLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for UserRoleLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Vec<Ulid>;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<(Ulid, Ulid)> =
			scuffle_utils::database::query("SELECT user_id, role_id FROM user_roles WHERE user_id = ANY($1)")
				.bind(keys)
				.build_query_scalar()
				.fetch_all(&self.db)
				.await
				.map_err(|e| {
					tracing::error!(err = %e, "failed to fetch user roles by user id");
				})?;

		Ok(results.into_iter().fold(HashMap::new(), |mut map, (user_id, role_id)| {
			map.entry(user_id).or_default().push(role_id);
			map
		}))
	}
}

struct UserBadgeLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl UserBadgeLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for UserBadgeLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Vec<Ulid>;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<(Ulid, Ulid)> =
			scuffle_utils::database::query("SELECT user_id, badge_id FROM user_badges WHERE user_id = ANY($1)")
				.bind(keys)
				.build_query_scalar()
				.fetch_all(&self.db)
				.await
				.map_err(|e| {
					tracing::error!(err = %e, "failed to fetch user badges by user id");
				})?;

		Ok(results.into_iter().fold(HashMap::new(), |mut map, (user_id, badge_id)| {
			map.entry(user_id).or_default().push(badge_id);
			map
		}))
	}
}

struct UserPaintLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl UserPaintLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for UserPaintLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Vec<Ulid>;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<(Ulid, Ulid)> =
			scuffle_utils::database::query("SELECT user_id, paint_id FROM user_paints WHERE user_id = ANY($1)")
				.bind(keys)
				.build_query_scalar()
				.fetch_all(&self.db)
				.await
				.map_err(|e| {
					tracing::error!(err = %e, "failed to fetch user paints by user id");
				})?;

		Ok(results.into_iter().fold(HashMap::new(), |mut map, (user_id, paint_id)| {
			map.entry(user_id).or_default().push(paint_id);
			map
		}))
	}
}

struct UserEmoteSetLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl UserEmoteSetLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for UserEmoteSetLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Vec<Ulid>;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<(Ulid, Ulid)> =
			scuffle_utils::database::query("SELECT user_id, emote_set_id FROM user_paints WHERE user_id = ANY($1)")
				.bind(keys)
				.build_query_scalar()
				.fetch_all(&self.db)
				.await
				.map_err(|e| {
					tracing::error!(err = %e, "failed to fetch user emote sets by user id");
				})?;

		Ok(results.into_iter().fold(HashMap::new(), |mut map, (user_id, emote_set_id)| {
			map.entry(user_id).or_default().push(emote_set_id);
			map
		}))
	}
}

struct InternalUserLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl InternalUserLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for InternalUserLoader {
	type Error = ();
	type Key = Ulid;
	type Value = User;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<Self::Value> = scuffle_utils::database::query("SELECT * FROM users WHERE id = ANY($1)")
			.bind(keys)
			.build_query_as()
			.fetch_all(&self.db)
			.await
			.map_err(|e| {
				tracing::error!(err = %e, "failed to fetch users by id");
			})?;

		Ok(results.into_iter().map(|r| (r.id, r)).collect())
	}
}

struct UserProductPurchaseLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl UserProductPurchaseLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for UserProductPurchaseLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Vec<ProductPurchase>;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<ProductPurchase> =
			scuffle_utils::database::query("SELECT * FROM product_purchases WHERE user_id = ANY($1) AND status = $2")
				.bind(keys)
				.bind(ProductPurchaseStatus::Completed)
				.build_query_as()
				.fetch_all(&self.db)
				.await
				.map_err(|e| {
					tracing::error!(err = %e, "failed to fetch product purchases by user id");
				})?;

		Ok(results.into_iter().fold(HashMap::new(), |mut map, pp| {
			map.entry(pp.user_id.unwrap()).or_default().push(pp);
			map
		}))
	}
}

struct UserProductLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl UserProductLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for UserProductLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Vec<UserProduct>;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<UserProduct> =
			scuffle_utils::database::query("SELECT * FROM user_products WHERE user_id = ANY($1)")
				.bind(keys)
				.build_query_as()
				.fetch_all(&self.db)
				.await
				.map_err(|e| {
					tracing::error!(err = %e, "failed to fetch user products by user id");
				})?;

		Ok(results.into_iter().fold(HashMap::new(), |mut map, sub| {
			map.entry(sub.user_id).or_default().push(sub);
			map
		}))
	}
}

impl UserLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> Self {
		Self {
			user_role_loader: UserRoleLoader::new(db.clone()),
			user_badge_loader: UserBadgeLoader::new(db.clone()),
			user_paint_loader: UserPaintLoader::new(db.clone()),
			user_emote_set_loader: UserEmoteSetLoader::new(db.clone()),
			user_loader: InternalUserLoader::new(db.clone()),
			user_product_purchase_loader: UserProductPurchaseLoader::new(db.clone()),
			user_products_loader: UserProductLoader::new(db.clone()),
			requests: Mutex::new(HashMap::new()),
		}
	}
}

impl UserLoader {
	pub async fn load_many(
		&self,
		global: &Arc<Global>,
		user_ids: impl IntoIterator<Item = Ulid>,
	) -> Result<HashMap<Ulid, User>, ()> {
		let user_ids = user_ids.into_iter().map(|id| self.load(global, id));

		let mut users = HashMap::new();

		for user in futures::future::join_all(user_ids).await {
			if let Some(user) = user? {
				users.insert(user.id, user);
			}
		}

		Ok(users)
	}

	pub async fn load(&self, global: &Arc<Global>, user_id: Ulid) -> Result<Option<User>, ()> {
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

		let _guard = token.done.drop_guard();

		let result = match self.internal_load_fn(global, user_id).await {
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

		self.requests.lock().await.remove(&user_id);

		result
	}

	async fn internal_load_fn(&self, global: &Arc<Global>, user_id: Ulid) -> anyhow::Result<Option<User>> {
		let Ok(Some(mut user)) = self.user_loader.load(user_id).await else {
			return Ok(None);
		};

		if user.entitled_cache.invalidated_at > chrono::Utc::now() {
			return Ok(Some(user));
		}

		let Ok(Some(role_ids)) = self.user_role_loader.load(user_id).await else {
			anyhow::bail!("failed to load user roles");
		};

		let Ok(Some(badge_ids)) = self.user_badge_loader.load(user_id).await else {
			anyhow::bail!("failed to load user badges");
		};

		let Ok(Some(paint_ids)) = self.user_paint_loader.load(user_id).await else {
			anyhow::bail!("failed to load user paints");
		};

		let Ok(Some(emote_set_ids)) = self.user_emote_set_loader.load(user_id).await else {
			anyhow::bail!("failed to load user emote sets");
		};

		let Ok(Some(product_purchaes)) = self.user_product_purchase_loader.load(user_id).await else {
			anyhow::bail!("failed to load user product purchases");
		};

		let product_purchaes =
			product_purchaes
				.into_iter()
				.fold(HashMap::<Ulid, Vec<ProductPurchase>>::new(), |mut map, pp| {
					map.entry(pp.product_id).or_default().push(pp);
					map
				});

		let Ok(Some(user_products)) = self.user_products_loader.load(user_id).await else {
			anyhow::bail!("failed to load user products");
		};

		let user_products = user_products.into_iter().fold(HashMap::new(), |mut map, sub| {
			map.entry(sub.product_id).or_insert(sub);
			map
		});

		let unique_product_ids = product_purchaes.keys().copied().collect::<Vec<_>>();

		let Ok(products) = global.product_by_id_loader().load_many(unique_product_ids).await else {
			anyhow::bail!("failed to load products");
		};

		user.entitled_cache = UserEntitledCache {
			role_ids,
			badge_ids,
			emote_set_ids,
			paint_ids,
			product_ids: Vec::new(),
			// 12 hours + 10%  jitter
			invalidated_at: chrono::Utc::now() + jitter(std::time::Duration::from_secs(12 * 60 * 60)),
		};

		for product in products.values() {
			let Some(purchases) = product_purchaes.get(&product.id) else {
				continue;
			};

			user.entitled_cache.product_ids.push(product.id);

			product
				.data
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

		let Ok(badge_ids) = global
			.role_badge_by_id_loader()
			.load_many(user.entitled_cache.role_ids.clone())
			.await
		else {
			anyhow::bail!("failed to load role badges");
		};

		user.entitled_cache.badge_ids.extend(badge_ids.into_values().flatten());

		let Ok(paint_ids) = global
			.role_paint_by_id_loader()
			.load_many(user.entitled_cache.role_ids.clone())
			.await
		else {
			anyhow::bail!("failed to load role paints");
		};

		user.entitled_cache.paint_ids.extend(paint_ids.into_values().flatten());

		let Ok(emote_set_ids) = global
			.role_emote_set_by_id_loader()
			.load_many(user.entitled_cache.role_ids.clone())
			.await
		else {
			anyhow::bail!("failed to load role emote sets");
		};

		user.entitled_cache
			.emote_set_ids
			.extend(emote_set_ids.into_values().flatten());

		scuffle_utils::database::query("UPDATE users SET entitled_cache_role_ids = $1, entitled_cache_badge_ids = $2, entitled_cache_emote_set_ids = $3, entitled_cache_paint_ids = $4, entitled_cache_invalidated_at = $5, entitled_cache_product_ids = $6 WHERE id = $7")
            .bind(&user.entitled_cache.role_ids)
            .bind(&user.entitled_cache.badge_ids)
            .bind(&user.entitled_cache.emote_set_ids)
            .bind(&user.entitled_cache.paint_ids)
            .bind(user.entitled_cache.invalidated_at)
            .bind(&user.entitled_cache.product_ids)
            .bind(user_id)
            .build()
            .execute(global.db())
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

	// count(purchases, #.was_gift and #.date >= '2021-12-01' and #.date <=
	// '2021-12-31') > 2 any(user_product.subscription_entries, #.status == 'Active'
	// and #.start <= '2021-12-01' and #.end >= '2021-12-01')

	let purchases = purchases
		.iter()
		.map(|pp| Purchase {
			date: chrono::Utc.timestamp_millis_opt(pp.id.timestamp_ms() as i64).unwrap(),
			was_gift: pp.data.was_gift,
			price: pp.data.price,
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

		let packed_end_at = up.created_at + total_time;

		let total_months = (packed_end_at.year() - up.created_at.year()) * 12 + packed_end_at.month() as i32
			- up.created_at.month() as i32
			+ if packed_end_at.day() < up.created_at.day() { -1 } else { 0 };

		let duraction = Duration {
			total_days: total_time.num_days(),
			total_years: total_time.num_days() / 365,
			total_months,
		};

		UserProduct {
			created_at: up.created_at,
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
