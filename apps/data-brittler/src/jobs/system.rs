use std::sync::Arc;

use mongodb::bson::{doc, to_bson};
use mongodb::options::UpdateOptions;
use shared::database::{
	Collection, EmotePermission, EmoteSetId, EmoteSetPermission, FeaturePermission, GlobalConfig, GlobalConfigAlerts, GlobalConfigId, Permissions, TicketPermission, UserPermission
};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types;

pub fn default_perms() -> Permissions {
	let mut perms = Permissions::default();

	perms.apply(EmotePermission::Upload.into());
	perms.apply(EmotePermission::Edit.into());

	perms.apply(EmoteSetPermission::Create.into());
	perms.apply(EmoteSetPermission::Delete.into());
	perms.apply(EmoteSetPermission::Edit.into());

	perms.apply(UserPermission::Login.into());
	perms.apply(UserPermission::Edit.into());

	perms.apply(FeaturePermission::UseBadge.into());
	perms.apply(FeaturePermission::UsePaint.into());
	perms.apply(FeaturePermission::UsePersonalEmoteSet.into());

	perms.apply(TicketPermission::Create.into());
	perms.apply(TicketPermission::Message.into());

	perms.emote_set_count_limit = Some(10);
	perms.emote_set_slots_limit = Some(600);

	perms
}

pub struct SystemJob {
	global: Arc<Global>,
}

impl Job for SystemJob {
	type T = types::System;

	const NAME: &'static str = "transfer_system";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		Ok(Self { global })
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("system")
	}

	async fn process(&mut self, system: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let emote_set_id: EmoteSetId = system.emote_set_id.into();

		match GlobalConfig::collection(self.global.target_db())
			.update_one(
				doc! {},
				doc! {
					"$addToSet": {
						"emote_set_ids": emote_set_id,
					},
					"$setOnInsert": {
						"_id": GlobalConfigId::nil(),
						"alerts": to_bson(&GlobalConfigAlerts::default()).unwrap(),
						"role_ids": [],
						"default_permissions": to_bson(&default_perms()).unwrap(),
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
