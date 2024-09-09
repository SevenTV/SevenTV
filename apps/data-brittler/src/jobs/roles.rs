use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

use bson::oid::ObjectId;
use shared::database::role::Role;
use shared::database::user::UserId;
use shared::database::MongoCollection;

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types;

pub struct RolesJob {
	global: Arc<Global>,
	ranks: HashSet<i32>,
}

impl Job for RolesJob {
	type T = types::Role;

	const NAME: &'static str = "transfer_roles";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping roles collection");
			Role::collection(global.target_db()).drop().await?;
			let indexes = Role::indexes();
			if !indexes.is_empty() {
				Role::collection(global.target_db()).create_indexes(indexes).await?;
			}
		}

		Ok(RolesJob {
			global,
			ranks: HashSet::new(),
		})
	}

	async fn collection(&self) -> Option<mongodb::Collection<Self::T>> {
		Some(self.global.source_db().collection("roles"))
	}

	async fn process(&mut self, role: Self::T) -> super::ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let id = role.id.into();

		let mut rank = role.position;

		while !self.ranks.insert(rank) {
			rank += 1;
		}

		match Role::collection(self.global.target_db())
			.insert_one(Role {
				id,
				permissions: role.to_new_permissions(),
				name: role.name,
				description: None,
				tags: vec![],
				hoist: role.color != 0,
				color: Some(role.color),
				rank,
				applied_rank: None,
				search_updated_at: None,
				created_by: UserId::nil(),
				updated_at: chrono::Utc::now(),
			})
			.await
		{
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}

	async fn finish(mut self) -> ProcessOutcome {
		// Insert a new role for translators
		self.process(types::Role {
			id: ObjectId::from_str("62f99d0ce46eb00e438a6984").unwrap(),
			name: "Translator".to_string(),
			position: 10,
			allowed: Default::default(),
			denied: Default::default(),
			color: 0,
		})
		.await
	}
}
