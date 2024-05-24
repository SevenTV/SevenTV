use std::sync::Arc;

use mongodb::bson::doc;
use mongodb::options::UpdateOptions;
use shared::database::{Collection, EmoteSetId, GlobalConfig, GlobalConfigId};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types;

pub struct SystemJob {
	global: Arc<Global>,
}

impl Job for SystemJob {
	type T = types::System;

	const NAME: &'static str = "transfer_system";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		Ok(Self { global })
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("system")
	}

	async fn process(&mut self, system: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let emote_set_id: EmoteSetId = system.emote_set_id.into();

		match GlobalConfig::collection(self.global.target_db())
			.update_one(
				doc! {},
				doc! {
					"$addToSet": {
						"emote_set_ids": emote_set_id,
					},
					"$setOnInsert": {
						"_id": GlobalConfigId::nil(),
						"alerts": [],
						"role_ids": [],
					},
				},
				UpdateOptions::builder().upsert(true).build(),
			)
			.await
		{
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}
}
