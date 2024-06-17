use std::sync::Arc;

use shared::database::{Collection, Permissions, UserBan, UserBanTemplate, UserBanTemplateId, UserPermission};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types;

pub struct BansJob {
	global: Arc<Global>,
	ban_role_id: UserBanTemplateId,
}

impl Job for BansJob {
	type T = types::Ban;

	const NAME: &'static str = "transfer_bans";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping user_bans collection");
			UserBan::collection(global.target_db()).drop(None).await?;
		}

		UserBanTemplate::collection(global.target_db()).drop(None).await?;

		let mut permissions = Permissions::default();
		permissions.user.deny(UserPermission::Login);

		let ban_role = UserBanTemplate {
			name: "Default Banned".to_string(),
			description: Some("Default role for banned users".to_string()),
			permissions,
			black_hole: true,
			..Default::default()
		};
		UserBanTemplate::collection(global.target_db())
			.insert_one(&ban_role, None)
			.await?;

		Ok(Self {
			global,
			ban_role_id: ban_role.id,
		})
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
					id: ban.id.into(),
					user_id: ban.victim_id.into(),
					created_by_id: Some(ban.actor_id.into()),
					reason: ban.reason,
					expires_at: Some(ban.expire_at.into_chrono()),
					template_id: self.ban_role_id,
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
