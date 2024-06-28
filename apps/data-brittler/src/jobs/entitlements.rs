use std::sync::Arc;

use fnv::FnvHashSet;
use mongodb::bson::doc;
use mongodb::options::InsertManyOptions;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::Collection;

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types::EntitlementData;
use crate::{error, types};

pub struct EntitlementsJob {
	global: Arc<Global>,
	edges: FnvHashSet<EntitlementEdge>,
}

impl Job for EntitlementsJob {
	type T = types::Entitlement;

	const NAME: &'static str = "transfer_entitlements";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping entitlements");
			EntitlementEdge::collection(global.target_db()).delete_many(doc! {}).await?;
		}

		Ok(Self {
			global,
			edges: FnvHashSet::default(),
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("entitlements")
	}

	async fn process(&mut self, entitlement: Self::T) -> ProcessOutcome {
		let Some(user_id) = entitlement.user_id else {
			return ProcessOutcome::default();
		};

		let to = match entitlement.data {
			EntitlementData::Badge { ref_id, .. } => EntitlementEdgeKind::Badge { badge_id: ref_id.into() },
			EntitlementData::Paint { ref_id, .. } => EntitlementEdgeKind::Paint { paint_id: ref_id.into() },
			EntitlementData::Role { ref_id } => EntitlementEdgeKind::Role { role_id: ref_id.into() },
			EntitlementData::EmoteSet { ref_id } => EntitlementEdgeKind::EmoteSet { emote_id: ref_id.into() },
			_ => return ProcessOutcome::default(),
		};

		self.edges.insert(EntitlementEdge {
			id: EntitlementEdgeId {
				from: EntitlementEdgeKind::User { user_id: user_id.into() },
				to,
				managed_by: None,
			},
		});

		ProcessOutcome::default()
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing entitlements job");

		let mut outcome = ProcessOutcome::default();

		match EntitlementEdge::collection(self.global.target_db())
			.insert_many(&self.edges)
			.with_options(InsertManyOptions::builder().ordered(false).build())
			.await
		{
			Ok(res) => {
				outcome.inserted_rows += res.inserted_ids.len() as u64;
				if res.inserted_ids.len() != self.edges.len() {
					outcome.errors.push(error::Error::InsertMany);
				}
			}
			Err(e) => outcome.errors.push(e.into()),
		}

		self.global.entitlement_job_token().cancel();

		outcome
	}
}
