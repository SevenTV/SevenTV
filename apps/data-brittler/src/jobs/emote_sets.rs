use std::sync::Arc;

use mongodb::options::InsertManyOptions;
use shared::database::{
	Collection, EmoteSet, EmoteSetEmote, EmoteSetEmoteFlag, EmoteSetEmoteId, EmoteSetFlags, EmoteSetKind,
};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub struct EmoteSetsJob {
	global: Arc<Global>,
	emote_sets: Vec<EmoteSet>,
	emote_set_emotes: Vec<EmoteSetEmote>,
}

impl Job for EmoteSetsJob {
	type T = types::EmoteSet;

	const NAME: &'static str = "transfer_emote_sets";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping emote_sets and emote_set_emotes collections");
			EmoteSet::collection(global.target_db()).drop(None).await?;
			EmoteSetEmote::collection(global.target_db()).drop(None).await?;
		}

		Ok(Self {
			global,
			emote_sets: vec![],
			emote_set_emotes: vec![],
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("emote_sets")
	}

	async fn process(&mut self, emote_set: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let kind = if emote_set.flags.contains(types::EmoteSetFlagModel::Personal) {
			EmoteSetKind::Personal
		} else {
			EmoteSetKind::Normal
		};

		let mut flags = EmoteSetFlags::none();
		if emote_set.immutable || emote_set.flags.contains(types::EmoteSetFlagModel::Immutable) {
			flags |= EmoteSetFlags::Immutable;
		}
		if emote_set.privileged || emote_set.flags.contains(types::EmoteSetFlagModel::Privileged) {
			flags |= EmoteSetFlags::Privileged;
		}

		self.emote_sets.push(EmoteSet {
			id: emote_set.id.into(),
			owner_id: Some(emote_set.owner_id.into()),
			name: emote_set.name,
			kind,
			tags: emote_set.tags,
			capacity: emote_set.capacity,
			flags,
		});

		for (emote_id, e) in emote_set
			.emotes
			.into_iter()
			.filter_map(|e| e)
			.filter_map(|e| e.id.map(|id| (id, e)))
		{
			let mut flags = EmoteSetEmoteFlag::none();

			if e.flags.contains(types::ActiveEmoteFlagModel::ZeroWidth) {
				flags |= EmoteSetEmoteFlag::ZeroWidth;
			}
			if e.flags.intersects(
				types::ActiveEmoteFlagModel::OverrideTwitchSubscriber
					| types::ActiveEmoteFlagModel::OverrideTwitchGlobal
					| types::ActiveEmoteFlagModel::OverrideBetterTTV,
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

			self.emote_set_emotes.push(EmoteSetEmote {
				id: EmoteSetEmoteId::new(),
				emote_set_id: emote_set.id.into(),
				emote_id: emote_id.into(),
				added_by_id: e.actor_id.map(Into::into),
				name: emote_name,
				flags,
			});
		}

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing emote sets job");

		let mut outcome = ProcessOutcome::default();

		let insert_options = InsertManyOptions::builder().ordered(false).build();
		let emote_sets = EmoteSet::collection(self.global.target_db());
		let emote_set_emotes = EmoteSetEmote::collection(self.global.target_db());

		let res = tokio::join!(
			emote_sets.insert_many(&self.emote_sets, insert_options.clone()),
			emote_set_emotes.insert_many(&self.emote_set_emotes, insert_options.clone()),
		);
		let res = vec![res.0, res.1]
			.into_iter()
			.zip(vec![self.emote_sets.len(), self.emote_set_emotes.len()]);

		for (res, len) in res {
			match res {
				Ok(res) => {
					outcome.inserted_rows += res.inserted_ids.len() as u64;
					if res.inserted_ids.len() != len {
						outcome.errors.push(error::Error::InsertMany);
					}
				}
				Err(e) => outcome.errors.push(e.into()),
			}
		}

		outcome
	}
}
