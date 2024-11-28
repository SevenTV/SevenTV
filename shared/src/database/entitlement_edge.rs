use std::collections::{HashMap, HashSet};
use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_batching::{DataLoader, DataLoaderFetcher};

use super::entitlement::EntitlementEdgeId;
use super::loader::dataloader::BatchLoad;
use super::queries::filter;
use crate::database::entitlement::{EntitlementEdge, EntitlementEdgeKind};
use crate::database::graph::GraphTraverse;
use crate::database::MongoCollection;

pub struct EntitlementEdgeInboundLoader {
	db: mongodb::Database,
	name: String,
}

impl EntitlementEdgeInboundLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			"EntitlementEdgeInboundLoader".to_string(),
			1000,
			50,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(
		db: mongodb::Database,
		name: String,
		batch_size: usize,
		concurrency: usize,
		delay: std::time::Duration,
	) -> DataLoader<Self> {
		DataLoader::new(Self { db, name }, batch_size, concurrency, delay)
	}
}

impl DataLoaderFetcher for EntitlementEdgeInboundLoader {
	type Key = EntitlementEdgeKind;
	type Value = Vec<EntitlementEdge>;

	async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

		let results: Vec<EntitlementEdge> = EntitlementEdge::collection(&self.db)
			.find(filter::filter! {
				EntitlementEdge {
					#[query(rename = "_id", flatten)]
					id: EntitlementEdgeId {
						#[query(serde, selector = "in")]
						to: keys,
					},
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})
			.ok()?;

		Some(results.into_iter().into_group_map_by(|edge| edge.id.to.clone()))
	}
}

pub struct EntitlementEdgeOutboundLoader {
	db: mongodb::Database,
	name: String,
}

impl EntitlementEdgeOutboundLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			"EntitlementEdgeOutboundLoader".to_string(),
			1000,
			50,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(
		db: mongodb::Database,
		name: String,
		batch_size: usize,
		concurrency: usize,
		delay: std::time::Duration,
	) -> DataLoader<Self> {
		DataLoader::new(Self { db, name }, batch_size, concurrency, delay)
	}
}

impl DataLoaderFetcher for EntitlementEdgeOutboundLoader {
	type Key = EntitlementEdgeKind;
	type Value = Vec<EntitlementEdge>;

	async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

		let results: Vec<EntitlementEdge> = EntitlementEdge::collection(&self.db)
			.find(filter::filter! {
				EntitlementEdge {
					#[query(rename = "_id", flatten)]
					id: EntitlementEdgeId {
						#[query(serde, selector = "in")]
						from: keys,
					},
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})
			.ok()?;

		Some(results.into_iter().into_group_map_by(|edge| edge.id.from.clone()))
	}
}

pub struct EntitlementEdgeGraphTraverse<'a> {
	pub inbound_loader: &'a DataLoader<EntitlementEdgeInboundLoader>,
	pub outbound_loader: &'a DataLoader<EntitlementEdgeOutboundLoader>,
}

impl GraphTraverse for EntitlementEdgeGraphTraverse<'_> {
	type Edge = EntitlementEdge;
	type Error = ();

	async fn fetch_edges(
		&self,
		direction: crate::database::graph::Direction,
		nodes: &[<Self::Edge as crate::database::graph::GraphEdge>::Key],
	) -> Result<Vec<Self::Edge>, Self::Error> {
		match direction {
			crate::database::graph::Direction::Inbound => Ok(self
				.inbound_loader
				.load_many(nodes.iter().cloned())
				.await?
				.into_values()
				.flatten()
				.collect()),
			crate::database::graph::Direction::Outbound => Ok(self
				.outbound_loader
				.load_many(nodes.iter().cloned())
				.await?
				.into_values()
				.flatten()
				.collect()),
		}
	}
}
