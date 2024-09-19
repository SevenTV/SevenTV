use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use shared::database::role::permissions::{Permissions, UserPermission};
use shared::database::user::ban::UserBan;

use super::{JobOutcome, ProcessOutcome};
use crate::global::Global;
use crate::types;

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub bans: &'a mut Vec<UserBan>,
}

pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("bans");

	let RunInput { global, bans } = input;

	let mut cursor = global
		.main_source_db
		.collection::<types::Ban>("bans")
		.find(bson::doc! {})
		.await
		.context("query")?;

	while let Some(ban) = cursor.next().await {
		match ban {
			Ok(ban) => {
				outcome += process(ProcessInput { ban, bans });
				outcome.processed_documents += 1;
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}

struct ProcessInput<'a> {
	pub ban: types::Ban,
	pub bans: &'a mut Vec<UserBan>,
}

fn process(input: ProcessInput<'_>) -> ProcessOutcome {
	let outcome = ProcessOutcome::default();

	let mut permissions = Permissions::default();
	permissions.user.deny(UserPermission::Login);

	let ProcessInput { ban, bans } = input;

	bans.push(UserBan {
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
	});

	outcome
}
