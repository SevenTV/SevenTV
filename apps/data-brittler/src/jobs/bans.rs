use std::sync::Arc;

use mongodb::bson::{doc, to_bson};
use shared::database::role::permissions::{Permissions, UserPermission};
use shared::database::user::ban::UserBan;
use shared::database::user::{User, UserId};
use shared::database::Collection;

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
		Ok(Self { global })
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("bans")
	}

	async fn process(&mut self, ban: Self::T) -> ProcessOutcome {
		// wait for users job to finish
		if self.global.config().should_run_users() {
			self.global.users_job_token().cancelled().await;
		}

		let mut outcome = ProcessOutcome::default();

		let mut permissions = Permissions::default();
		permissions.user.deny(UserPermission::Login);

		let user_ban = UserBan {
			id: ban.id.into(),
			created_by_id: ban.actor_id.into(),
			reason: ban.reason,
			tags: vec![],
			expires_at: Some(ban.expire_at.into_chrono()),
			removed: None,
			permissions,
			template_id: None,
		};

		let user_ban = to_bson(&user_ban).expect("failed to serialize ban");

		let user_id: UserId = ban.victim_id.into();

		match User::collection(self.global.target_db())
			.update_one(
				doc! {
					"_id": user_id,
				},
				doc! {
					"$push": {
						"bans": user_ban,
					},
				},
			)
			.await
		{
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}
}
