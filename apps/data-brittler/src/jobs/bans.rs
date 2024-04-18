use std::sync::Arc;

use shared::database::{Collection, UserBan};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types;

pub struct BansJob {
	global: Arc<Global>,
}

impl Job for BansJob {
	type T = types::Ban;

	const NAME: &'static str = "transfer_bans";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping user_bans collection");
			UserBan::collection(global.target_db()).drop(None).await?;
		}

		Ok(Self { global })
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("bans")
	}

	async fn process(&mut self, ban: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		// TODO: effects

		match UserBan::collection(self.global.target_db())
			.insert_one(
				UserBan {
					id: ban.id,
					user_id: ban.victim_id,
					created_by_id: Some(ban.actor_id),
					reason: ban.reason,
					expires_at: Some(ban.expire_at.into_chrono()),
				},
				None,
			)
			.await
		{
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}
}
