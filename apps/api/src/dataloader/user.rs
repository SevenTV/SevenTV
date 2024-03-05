use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use rand::Rng;
use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use tokio::sync::{Mutex, OnceCell};
use tokio_util::sync::CancellationToken;
use ulid::Ulid;

use crate::database::{
	Product, ProductEntitlement, ProductPurchase, ProductPurchaseStatus, ProductSubscription, ProductSubscriptionStatus,
	User, UserEntitledCache,
};
use crate::global::Global;

pub struct UserLoader {
	user_role_loader: DataLoader<UserRoleLoader>,
	user_loader: DataLoader<InternalUserLoader>,
	user_product_purchase_loader: DataLoader<UserProductPurchaseLoader>,
	user_subscription_loader: DataLoader<UserSubscriptionLoader>,
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
			map.entry(user_id).or_insert_with(Vec::new).push(role_id);
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
			map.entry(pp.user_id.unwrap()).or_insert_with(Vec::new).push(pp);
			map
		}))
	}
}

struct UserSubscriptionLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl UserSubscriptionLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for UserSubscriptionLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Vec<ProductSubscription>;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<ProductSubscription> =
			scuffle_utils::database::query("SELECT * FROM product_subscriptions WHERE user_id = ANY($1) AND status = $2")
				.bind(keys)
				.bind(ProductSubscriptionStatus::Active)
				.build_query_as()
				.fetch_all(&self.db)
				.await
				.map_err(|e| {
					tracing::error!(err = %e, "failed to fetch product subscriptions by user id");
				})?;

		Ok(results.into_iter().fold(HashMap::new(), |mut map, sub| {
			map.entry(sub.user_id).or_insert_with(Vec::new).push(sub);
			map
		}))
	}
}

impl UserLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> Self {
		Self {
			user_role_loader: UserRoleLoader::new(db.clone()),
			user_loader: InternalUserLoader::new(db.clone()),
			user_product_purchase_loader: UserProductPurchaseLoader::new(db.clone()),
			user_subscription_loader: UserSubscriptionLoader::new(db.clone()),
			requests: Mutex::new(HashMap::new()),
		}
	}
}

impl UserLoader {
	pub async fn load(&self, global: &Arc<Global>, user_id: Ulid) -> Result<User, ()> {
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
				return Ok(token.result.get().cloned().ok_or(())?);
			}

			token
		};

		let _guard = token.done.drop_guard();

		let result = match self.internal_load_fn(global, user_id).await {
			Ok(Some(user)) => {
				token.result.set(user.clone()).ok();
				Ok(user)
			}
			Ok(None) => Err(()),
			Err(err) => {
				tracing::error!("failed to load user entitlements: {}", err);
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
			return Ok(None);
		};

		let Ok(Some(product_purchaes)) = self.user_product_purchase_loader.load(user_id).await else {
			return Ok(None);
		};

		let product_purchaes = product_purchaes.into_iter().fold(HashMap::new(), |mut map, pp| {
			map.entry(pp.product_id).or_insert_with(Vec::new).push(pp);
			map
		});

		let Ok(Some(subscriptions)) = self.user_subscription_loader.load(user_id).await else {
			return Ok(None);
		};

		let subscriptions = subscriptions.into_iter().fold(HashMap::new(), |mut map, sub| {
			map.entry(sub.product_id).or_insert(sub);
			map
		});

		let unique_product_ids = product_purchaes.keys().copied().collect::<Vec<_>>();

		let Ok(products) = global.product_by_id_loader().load_many(unique_product_ids).await else {
			return Ok(None);
		};

		user.entitled_cache = UserEntitledCache {
			role_ids,
			badge_ids: Vec::new(),
			emote_set_ids: Vec::new(),
			paint_ids: Vec::new(),
			product_ids: Vec::new(),
			invalidated_at: chrono::Utc::now() + jitter(std::time::Duration::from_secs(12 * 60 * 60)), /* 12 hours + 10%
			                                                                                            * jitter */
		};

		for product in products.values() {
			let Some(purchases) = product_purchaes.get(&product.id) else {
				continue;
			};

			user.entitled_cache.product_ids.push(product.id);

			for entitlement_group in &product.data.entitlement_groups {
				if entitlement_group
					.condition
					.as_ref()
					.map(|c| c.evaluate(&purchases, subscriptions.get(&product.id)))
					.unwrap_or(true)
				{
					for entitlement in &entitlement_group.entitlements {
						match entitlement {
							ProductEntitlement::Role(role_id) => {
								user.entitled_cache.role_ids.push(*role_id);
							}
							ProductEntitlement::Badge(badge_id) => {
								user.entitled_cache.badge_ids.push(*badge_id);
							}
							ProductEntitlement::Paint(paint_id) => {
								user.entitled_cache.paint_ids.push(*paint_id);
							}
							ProductEntitlement::EmoteSet(emote_set_id) => {
								user.entitled_cache.emote_set_ids.push(*emote_set_id);
							}
						}
					}
				}
			}
		}

		let Ok(badge_ids) = global
			.role_badge_by_id_loader()
			.load_many(user.entitled_cache.role_ids.clone())
			.await
		else {
			return Ok(None);
		};

		user.entitled_cache.badge_ids.extend(badge_ids.into_values().flatten());

		let Ok(paint_ids) = global
			.role_paint_by_id_loader()
			.load_many(user.entitled_cache.role_ids.clone())
			.await
		else {
			return Ok(None);
		};

		user.entitled_cache.paint_ids.extend(paint_ids.into_values().flatten());

		let Ok(emote_set_ids) = global
			.role_emote_set_by_id_loader()
			.load_many(user.entitled_cache.role_ids.clone())
			.await
		else {
			return Ok(None);
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
