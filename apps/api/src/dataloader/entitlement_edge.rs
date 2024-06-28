use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use mongodb::bson::to_bson;
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeKind};
use shared::database::graph::GraphTraverse;
use shared::database::Collection;

pub struct EntitlementEdgeInboundLoader {
	db: mongodb::Database,
}

impl EntitlementEdgeInboundLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("EntitlementEdgeInboundLoader", Self { db })
	}
}

impl Loader for EntitlementEdgeInboundLoader {
	type Error = ();
	type Key = EntitlementEdgeKind;
	type Value = Vec<EntitlementEdge>;

	#[tracing::instrument(name = "UserByIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<EntitlementEdge> = EntitlementEdge::collection(&self.db)
			.find(doc! {
				"_id.to": {
					"$in": to_bson(&keys).unwrap(),
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
}

impl EntitlementEdgeOutboundLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("EntitlementEdgeOutboundLoader", Self { db })
	}
}

impl Loader for EntitlementEdgeOutboundLoader {
	type Error = ();
	type Key = EntitlementEdgeKind;
	type Value = Vec<EntitlementEdge>;

	#[tracing::instrument(name = "UserByIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<EntitlementEdge> = EntitlementEdge::collection(&self.db)
			.find(doc! {
				"_id.from": {
					"$in": to_bson(&keys).unwrap(),
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
		direction: shared::database::graph::Direction,
		nodes: &[<Self::Edge as shared::database::graph::GraphEdge>::Key],
	) -> Result<Vec<Self::Edge>, Self::Error> {
		match direction {
			shared::database::graph::Direction::Inbound => Ok(self
				.inbound_loader
				.load_many(nodes.into_iter().cloned())
				.await?
				.into_values()
				.flatten()
				.collect()),
			shared::database::graph::Direction::Outbound => Ok(self
				.outbound_loader
				.load_many(nodes.into_iter().cloned())
				.await?
				.into_values()
				.flatten()
				.collect()),
		}
	}
}
