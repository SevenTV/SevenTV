use std::sync::Arc;

use mongodb::bson::{doc, to_bson};
use mongodb::options::UpdateOptions;
use shared::database::global::{GlobalConfig, GlobalConfigAlerts};
use shared::database::role::{Role, RoleId};
use shared::database::{Collection, Id};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types;

pub struct RolesJob {
	global: Arc<Global>,
	all_roles: Vec<(RoleId, i16)>,
}

impl Job for RolesJob {
	type T = types::Role;

	const NAME: &'static str = "transfer_roles";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping roles collection");
			Role::collection(global.target_db()).drop(None).await?;
		}

		Ok(RolesJob {
			global,
			all_roles: Vec::new(),
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("roles")
	}

	async fn process(&mut self, role: Self::T) -> super::ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let id = role.id.into();

		let priority = role.position.try_into().unwrap_or(i16::MAX);
		self.all_roles.push((id, priority));

		match Role::collection(self.global.target_db())
			.insert_one(
				Role {
					id,
					permissions: role.to_new_permissions(),
					name: role.name,
					description: None,
					tags: vec![],
					hoist: false,
					color: Some(role.color),
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

	async fn finish(mut self) -> ProcessOutcome {
		self.all_roles.sort_by_key(|(_, p)| *p);

		let role_ids: Vec<RoleId> = self.all_roles.into_iter().map(|(id, _)| id).collect();

		let mut outcome = ProcessOutcome::default();

		match GlobalConfig::collection(self.global.target_db())
			.update_one(
				doc! {},
				doc! {
					"$addToSet": {
						"role_ids": role_ids,
					},
					"$setOnInsert": {
						"_id": Id::<()>::nil().as_uuid(),
						"alerts": to_bson(&GlobalConfigAlerts::default()).unwrap(),
						"emote_set_ids": [],
						"automod_rule_ids": [],
						"normal_emote_set_slot_capacity": 600,
						"personal_emote_set_slot_capacity": 5,
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
