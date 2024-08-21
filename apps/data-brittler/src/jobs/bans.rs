use std::sync::Arc;

use mongodb::bson::doc;
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{Permissions, UserPermission};
use shared::database::user::ban::UserBan;
use shared::database::user::{User, UserId};
use shared::database::MongoCollection;

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
			tracing::info!("dropping user_bans collections");
			UserBan::collection(global.target_db()).untyped().drop().await?;
			let indexes = UserBan::indexes();
			if !indexes.is_empty() {
				UserBan::collection(global.target_db())
					.untyped()
					.create_indexes(indexes)
					.await?;
			}
		}

		Ok(Self { global })
	}

	async fn collection(&self) -> Option<mongodb::Collection<Self::T>> {
		Some(self.global.source_db().collection("bans"))
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
			user_id: ban.victim_id.into(),
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		};

		let user_id: UserId = ban.victim_id.into();

		match User::collection(self.global.target_db())
			.update_one(
				filter::filter! {
					User {
						#[query(rename = "_id")]
						id: user_id,
					}
				},
				update::update! {
					#[query(set)]
					User {
						has_bans: true,
					}
				},
			)
			.await
		{
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => outcome.errors.push(e.into()),
		}

		match UserBan::collection(self.global.target_db()).insert_one(user_ban).await {
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}
}
