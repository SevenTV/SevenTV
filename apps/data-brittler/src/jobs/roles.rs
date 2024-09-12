use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

use bson::oid::ObjectId;
use shared::database::role::permissions::{EmotePermission, EmoteSetPermission, Permissions, TicketPermission, UserPermission};
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

		let mut ranks = HashSet::new();

		ranks.insert(0); // Default
		ranks.insert(5); // Subscriber
		ranks.insert(6); // Contributor

		Ok(RolesJob {
			global,
			ranks,
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

		// if this is the default role
		if role.id != "62b48deb791a15a25c2a0354".parse().unwrap() && role.id != "6076a86b09a4c63a38ebe801".parse().unwrap() {
			match Role::collection(self.global.target_db())
				.insert_one(Role {
					id,
					permissions: role.to_new_permissions(),
					name: role.name,
					description: None,
					tags: vec![],
					hoist: role.color != 0,
					color: (role.color != 0).then_some(role.color),
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
		}

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let mut roles = vec![];

		// Insert default role
		let mut default_permissions = Permissions::default();
		default_permissions.emote.allow(EmotePermission::Upload);
		default_permissions.emote.allow(EmotePermission::Edit);
		default_permissions.emote.allow(EmotePermission::Delete);

		default_permissions.emote_set.allow(EmoteSetPermission::Manage);

		default_permissions.user.allow(UserPermission::Login);
		default_permissions.user.allow(UserPermission::InviteEditors);
		default_permissions.user.allow(UserPermission::UseBadge);
		default_permissions.user.allow(UserPermission::UsePaint);

		default_permissions.ticket.allow(TicketPermission::Create);
		default_permissions.ticket.allow(TicketPermission::Message);

		default_permissions.emote_moderation_request_priority = Some(1);
		default_permissions.emote_moderation_request_limit = Some(10);
		default_permissions.emote_set_capacity = Some(1000);

		roles.push(Role {
			id: ObjectId::from_str("62b48deb791a15a25c2a0354").unwrap().into(),
			permissions: default_permissions,
			name: "Default".to_string(),
			description: None,
			tags: vec![],
			hoist: false,
			color: None,
			rank: 0,
			applied_rank: None,
			search_updated_at: None,
			created_by: UserId::nil(),
			updated_at: chrono::Utc::now(),
		});

		let mut sub_permissions = Permissions::default();
		sub_permissions.user.allow(UserPermission::UseCustomProfilePicture);
		sub_permissions.user.allow(UserPermission::UsePersonalEmoteSet);
		sub_permissions.personal_emote_set_capacity = Some(5);

		roles.push(Role {
			id: ObjectId::from_str("6076a86b09a4c63a38ebe801").unwrap().into(),
			permissions: Permissions::default(),
			name: "Subscriber".to_string(),
			description: None,
			tags: vec![],
			hoist: false,
			color: Some(-5635841),
			rank: 5,
			applied_rank: None,
			search_updated_at: None,
			created_by: UserId::nil(),
			updated_at: chrono::Utc::now(),
		});

		roles.push(Role {
			id: ObjectId::from_str("62f99d0ce46eb00e438a6984").unwrap().into(),
			permissions: Permissions::default(),
			name: "Translator".to_string(),
			description: None,
			tags: vec![],
			hoist: false,
			color: None,
			rank: 6,
			applied_rank: None,
			search_updated_at: None,
			created_by: UserId::nil(),
			updated_at: chrono::Utc::now(),
		});

		match Role::collection(self.global.target_db()).insert_many(&roles).await {
			Ok(r) if r.inserted_ids.len() == roles.len() => {
				outcome.inserted_rows += r.inserted_ids.len() as u64;
			}
			Ok(_) => outcome = outcome.with_error(crate::error::Error::InsertMany),
			Err(e) => outcome = outcome.with_error(e),
		}

		outcome
	}
}
