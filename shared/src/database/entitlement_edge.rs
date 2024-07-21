use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_foundations::batcher::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::batcher::BatcherConfig;
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;

use super::entitlement::EntitlementEdgeId;
use super::queries::filter;
use crate::database::entitlement::{EntitlementEdge, EntitlementEdgeKind};
use crate::database::graph::GraphTraverse;
use crate::database::MongoCollection;

pub struct EntitlementEdgeInboundLoader {
	db: mongodb::Database,
}

impl EntitlementEdgeInboundLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for EntitlementEdgeInboundLoader {
	type Key = EntitlementEdgeKind;
	type Value = Vec<EntitlementEdge>;

	#[tracing::instrument(skip_all, fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<EntitlementEdge> = EntitlementEdge::collection(&self.db)
			.find(filter::filter! {
				EntitlementEdge {
					#[filter(rename = "_id", flatten)]
					id: EntitlementEdgeId {
						#[filter(selector = "in")]
						to: keys,
					},
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().into_group_map_by(|edge| edge.id.to.clone()))
	}
}

pub struct EntitlementEdgeOutboundLoader {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl EntitlementEdgeOutboundLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: format!("EntitlementEdgeOutboundLoader"),
				concurrency: 50,
				max_batch_size: 1_000,
				sleep_duration: std::time::Duration::from_millis(5),
			},
		)
	}

	pub fn new_with_config(db: mongodb::Database, config: BatcherConfig) -> DataLoader<Self> {
		DataLoader::new(Self { db, config })
	}
}

impl Loader for EntitlementEdgeOutboundLoader {
	type Key = EntitlementEdgeKind;
	type Value = Vec<EntitlementEdge>;

	fn config(&self) -> BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let results: Vec<EntitlementEdge> = EntitlementEdge::collection(&self.db)
			.find(filter::filter! {
				EntitlementEdge {
					#[filter(rename = "_id", flatten)]
					id: EntitlementEdgeId {
						#[filter(selector = "in")]
						from: keys,
					},
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().into_group_map_by(|edge| edge.id.from.clone()))
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
				.load_many(nodes.into_iter().cloned())
				.await?
				.into_values()
				.flatten()
				.collect()),
			crate::database::graph::Direction::Outbound => Ok(self
				.outbound_loader
				.load_many(nodes.into_iter().cloned())
				.await?
				.into_values()
				.flatten()
				.collect()),
		}
	}
}
