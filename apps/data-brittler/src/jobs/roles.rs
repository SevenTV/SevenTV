use std::sync::Arc;

use mongodb::bson::doc;
use shared::database::role::Role;
use shared::database::Collection;

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types;

pub struct RolesJob {
	global: Arc<Global>,
}

impl Job for RolesJob {
	type T = types::Role;

	const NAME: &'static str = "transfer_roles";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping roles collection");
			Role::collection(global.target_db()).delete_many(doc! {}).await?;
		}

		Ok(RolesJob {
			global,
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("roles")
	}

	async fn process(&mut self, role: Self::T) -> super::ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let id = role.id.into();

		let rank = role.position.try_into().unwrap_or(i16::MAX);

		match Role::collection(self.global.target_db())
			.insert_one(Role {
				id,
				permissions: role.to_new_permissions(),
				name: role.name,
				description: None,
				tags: vec![],
				hoist: false,
				color: Some(role.color),
				rank: rank as i32,
			})
			.await
		{
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}
}
