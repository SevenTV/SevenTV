use std::sync::Arc;

use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::options::UpdateOptions;
use shared::database::{Collection, EmoteSetId, GlobalConfig, Role};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types;

pub struct RolesJob {
	global: Arc<Global>,
	all_roles: Vec<(ObjectId, i16)>,
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

		let priority = role.position.try_into().unwrap_or(i16::MAX);
		self.all_roles.push((role.id, priority));

		match Role::collection(self.global.target_db())
			.insert_one(
				Role {
					id: role.id.into(),
					badge_ids: vec![],
					paint_ids: vec![],
					emote_set_ids: vec![],
					permissions: role.to_new_permissions(),
					name: role.name,
					description: None,
					hoist: false,
					color: role.color,
					tags: vec![],
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
		self.all_roles.sort_by_cached_key(|(_, p)| *p);

		let role_ids: Vec<EmoteSetId> = self.all_roles.into_iter().map(|(id, _)| id.into()).collect();

		let mut outcome = ProcessOutcome::default();

		match GlobalConfig::collection(self.global.target_db())
			.update_one(
				doc! {},
				doc! {
					"$set": {
						"role_ids": role_ids,
					},
					"$setOnInsert": {
						"alerts": [],
						"emote_set_ids": [],
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
