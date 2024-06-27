use std::sync::Arc;

use mongodb::bson::doc;
use mongodb::options::InsertManyOptions;
use shared::database::emote_set::{EmoteSet, EmoteSetEmote, EmoteSetEmoteFlag, EmoteSetKind};
use shared::database::Collection;
use shared::old_types::{ActiveEmoteFlagModel, EmoteSetFlagModel};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub struct EmoteSetsJob {
	global: Arc<Global>,
	emote_sets: Vec<EmoteSet>,
}

impl Job for EmoteSetsJob {
	type T = types::EmoteSet;

	const NAME: &'static str = "transfer_emote_sets";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping emote_sets and emote_set_emotes collections");
			EmoteSet::collection(global.target_db()).delete_many(doc! {}, None).await?;
		}

		Ok(Self {
			global,
			emote_sets: vec![],
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("emote_sets")
	}

	async fn process(&mut self, emote_set: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let kind = if emote_set.flags.contains(EmoteSetFlagModel::Personal) {
			EmoteSetKind::Personal
		} else {
			EmoteSetKind::Normal
		};

		let mut emotes = vec![];

		for (emote_id, e) in emote_set.emotes.into_iter().flatten().filter_map(|e| e.id.map(|id| (id, e))) {
			let mut flags = EmoteSetEmoteFlag::none();

			if e.flags.contains(ActiveEmoteFlagModel::ZeroWidth) {
				flags |= EmoteSetEmoteFlag::ZeroWidth;
			}
			if e.flags.intersects(
				ActiveEmoteFlagModel::OverrideTwitchSubscriber
					| ActiveEmoteFlagModel::OverrideTwitchGlobal
					| ActiveEmoteFlagModel::OverrideBetterTTV,
			) {
				flags |= EmoteSetEmoteFlag::OverrideConflicts;
			}

			let Some(emote_name) = e.name else {
				outcome.errors.push(error::Error::EmoteSetEmoteNoName {
					emote_set_id: emote_set.id,
					emote_id,
				});
				continue;
			};

			emotes.push(EmoteSetEmote {
				id: emote_id.into(),
				alias: emote_name,
				added_at: e.timestamp.map(|t| t.into_chrono()).unwrap_or_default(),
				flags,
				added_by_id: e.actor_id.map(Into::into),
				origin_set_id: None,
			});
		}

		self.emote_sets.push(EmoteSet {
			id: emote_set.id.into(),
			name: emote_set.name,
			description: None,
			tags: emote_set.tags,
			emotes,
			capacity: Some(emote_set.capacity),
			owner_id: Some(emote_set.owner_id.into()),
			origin_config: None,
			kind,
		});

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing emote sets job");

		let mut outcome = ProcessOutcome::default();

		match EmoteSet::collection(self.global.target_db())
			.insert_many(&self.emote_sets, InsertManyOptions::builder().ordered(false).build())
			.await
		{
			Ok(res) => {
				outcome.inserted_rows += res.inserted_ids.len() as u64;
				if res.inserted_ids.len() != self.emote_sets.len() {
					outcome.errors.push(error::Error::InsertMany);
				}
			}
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}
}
