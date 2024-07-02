use std::sync::Arc;

use shared::database::emote_set::EmoteSetId;
use shared::database::global::{GlobalConfig, GlobalConfigAlerts};
use shared::database::MongoCollection;

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
		if global.config().truncate {
			GlobalConfig::collection(global.target_db()).drop().await?;
			let indexes = GlobalConfig::indexes();
			if !indexes.is_empty() {
				GlobalConfig::collection(global.target_db()).create_indexes(indexes).await?;
			}
		}

		Ok(Self { global })
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("system")
	}

	async fn process(&mut self, system: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let emote_set_id: EmoteSetId = system.emote_set_id.into();

		let config = GlobalConfig {
			id: (),
			alerts: GlobalConfigAlerts::default(),
			emote_set_id,
			automod_rule_ids: vec![],
		};
		match GlobalConfig::collection(self.global.target_db()).insert_one(config).await {
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}
}
