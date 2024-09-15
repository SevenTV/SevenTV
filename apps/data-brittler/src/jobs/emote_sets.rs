use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use shared::database::emote::EmoteId;
use shared::database::emote_set::{EmoteSet, EmoteSetEmote, EmoteSetEmoteFlag, EmoteSetKind};
use shared::database::queries::filter;
use shared::database::MongoCollection;
use shared::old_types::{ActiveEmoteFlagModel, EmoteSetFlagModel};

use super::{JobOutcome, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub emote_sets: &'a mut Vec<EmoteSet>,
	pub filter: Box<dyn Fn(&EmoteId) -> bool + 'a>,
}

pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("emote_sets");

	let RunInput {
		global,
		emote_sets,
		filter,
	} = input;

	let mut cursor = global
		.source_db()
		.collection::<types::EmoteSet>("emote_sets")
		.find(bson::doc! {})
		.await
		.context("query")?;

	while let Some(emote_set) = cursor.next().await {
		match emote_set {
			Ok(emote_set) => {
				outcome += process(ProcessInput {
					global,
					emote_sets,
					emote_set,
					filter: &filter,
				});
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}

struct ProcessInput<'a> {
	global: &'a Arc<Global>,
	emote_sets: &'a mut Vec<EmoteSet>,
	emote_set: types::EmoteSet,
	filter: &'a Box<dyn Fn(&EmoteId) -> bool + 'a>,
}

fn process(input: ProcessInput<'_>) -> ProcessOutcome {
	let mut outcome = ProcessOutcome::default();

	let ProcessInput {
		global,
		emote_sets,
		emote_set,
		filter,
	} = input;

	let kind = if emote_set.flags.contains(EmoteSetFlagModel::Personal) {
		EmoteSetKind::Personal
	} else {
		EmoteSetKind::Normal
	};

	let mut emotes = vec![];

	for (emote_id, e) in emote_set.emotes.into_iter().flatten().filter_map(|e| e.id.map(|id| (id, e))) {
		if !filter(&emote_id.into()) {
			continue;
		}

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

	emote_sets.push(EmoteSet {
		id: emote_set.id.into(),
		name: emote_set.name,
		description: None,
		tags: emote_set.tags,
		emotes,
		capacity: Some(emote_set.capacity),
		owner_id: Some(emote_set.owner_id.into()),
		origin_config: None,
		kind,
		emotes_changed_since_reindex: true,
		search_updated_at: None,
		updated_at: chrono::Utc::now(),
	});

	outcome
}

pub async fn skip(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("emote_sets");

	let RunInput { global, emote_sets, .. } = input;

	let mut cursor = EmoteSet::collection(global.target_db())
		.find(filter::filter! {
			EmoteSet {}
		})
		.await
		.context("query")?;

	while let Some(emote_set) = cursor.next().await {
		match emote_set {
			Ok(emote_set) => {
				emote_sets.push(emote_set);
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}
