use std::collections::HashMap;
use std::sync::Arc;

use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use ulid::Ulid;

use crate::database::Role;

pub struct RoleByIdLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl RoleByIdLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for RoleByIdLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Role;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<Self::Value> = scuffle_utils::database::query("SELECT * FROM roles WHERE id = ANY($1)")
			.bind(keys)
			.build_query_as()
			.fetch_all(&self.db)
			.await
			.map_err(|e| {
				tracing::error!(err = %e, "failed to fetch roles by id");
			})?;

		Ok(results.into_iter().map(|r| (r.id, r)).collect())
	}
}

pub struct RoleBadgeByIdLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl RoleBadgeByIdLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for RoleBadgeByIdLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Vec<Ulid>;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<(Ulid, Ulid)> =
			scuffle_utils::database::query("SELECT role_id, badge_id FROM role_badges WHERE role_id = ANY($1)")
				.bind(keys)
				.build_query_scalar()
				.fetch_all(&self.db)
				.await
				.map_err(|e| {
					tracing::error!(err = %e, "failed to fetch role badges by id");
				})?;

		Ok(results.into_iter().fold(HashMap::new(), |mut acc, (role_id, badge_id)| {
			acc.entry(role_id).or_default().push(badge_id);
			acc
		}))
	}
}

pub struct RolePaintByIdLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl RolePaintByIdLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for RolePaintByIdLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Vec<Ulid>;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<(Ulid, Ulid)> =
			scuffle_utils::database::query("SELECT role_id, paint_id FROM role_paints WHERE role_id = ANY($1)")
				.bind(keys)
				.build_query_scalar()
				.fetch_all(&self.db)
				.await
				.map_err(|e| {
					tracing::error!(err = %e, "failed to fetch role paints by id");
				})?;

		Ok(results.into_iter().fold(HashMap::new(), |mut acc, (role_id, paint_id)| {
			acc.entry(role_id).or_default().push(paint_id);
			acc
		}))
	}
}

pub struct RoleEmoteSetByIdLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl RoleEmoteSetByIdLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for RoleEmoteSetByIdLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Vec<Ulid>;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<(Ulid, Ulid)> =
			scuffle_utils::database::query("SELECT role_id, emote_set_id FROM role_emote_sets WHERE role_id = ANY($1)")
				.bind(keys)
				.build_query_scalar()
				.fetch_all(&self.db)
				.await
				.map_err(|e| {
					tracing::error!(err = %e, "failed to fetch role emote_sets by id");
				})?;

		Ok(results.into_iter().fold(HashMap::new(), |mut acc, (role_id, paint_id)| {
			acc.entry(role_id).or_default().push(paint_id);
			acc
		}))
	}
}
