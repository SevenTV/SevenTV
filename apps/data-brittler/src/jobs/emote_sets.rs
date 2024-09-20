use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use shared::database::emote::{Emote, EmoteId};
use shared::database::emote_set::{EmoteSet, EmoteSetEmote, EmoteSetEmoteFlag, EmoteSetId, EmoteSetKind};
use shared::database::user::{User, UserId};
use shared::old_types::{ActiveEmoteFlagModel, EmoteSetFlagModel};

use super::{JobOutcome, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub emote_sets: &'a mut HashMap<EmoteSetId, EmoteSet>,
	pub rankings: &'a mut HashMap<EmoteId, i32>,
	pub users: &'a mut HashMap<UserId, User>,
	pub emotes: &'a HashMap<EmoteId, Emote>,
}

pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("emote_sets");

	let RunInput {
		global,
		emote_sets,
		emotes,
		users,
		rankings,
	} = input;

	let mut cursor = global
		.main_source_db
		.collection::<types::EmoteSet>("emote_sets")
		.find(bson::doc! {})
		.await
		.context("query")?;

	while let Some(emote_set) = cursor.next().await {
		match emote_set {
			Ok(emote_set) => {
				outcome += process(ProcessInput {
					emote_sets,
					emote_set,
					users,
					emotes,
				});
				outcome.processed_documents += 1;
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}
	}

	users.values_mut().for_each(|user| {
		if let Some(emote_set_id) = user.style.active_emote_set_id {
			let Some(emote_set) = emote_sets.get(&emote_set_id) else {
				user.style.active_emote_set_id = None;
				return;
			};

			user.cached.active_emotes = emote_set
				.emotes
				.iter()
				.map(|e| {
					*rankings.entry(e.id).or_default() += 1;
					e.id
				})
				.collect();

			user.cached.emote_set_id = Some(emote_set.id);
		}
	});

	Ok(outcome)
}

struct ProcessInput<'a> {
	emote_sets: &'a mut HashMap<EmoteSetId, EmoteSet>,
	emote_set: types::EmoteSet,
	users: &'a mut HashMap<UserId, User>,
	emotes: &'a HashMap<EmoteId, Emote>,
}

fn process(input: ProcessInput<'_>) -> ProcessOutcome {
	let mut outcome = ProcessOutcome::default();

	let ProcessInput {
		emote_sets,
		emote_set,
		emotes,
		users,
	} = input;

	let kind = match (
		emote_set.flags.contains(EmoteSetFlagModel::Personal),
		emote_set.flags.contains(EmoteSetFlagModel::Privileged),
	) {
		(true, false) => EmoteSetKind::Personal,
		(false, false) => EmoteSetKind::Normal,
		(true, true) => EmoteSetKind::Special,
		(false, true) => EmoteSetKind::Global,
	};

	match kind {
		EmoteSetKind::Personal => {
			let Some(user) = users.get_mut(&emote_set.owner_id.into()) else {
				return ProcessOutcome::default();
			};

			user.style.personal_emote_set_id = Some(emote_set.id.into());
		}
		EmoteSetKind::Normal => {
			if !users.contains_key(&emote_set.owner_id.into()) {
				return ProcessOutcome::default();
			}
		}
		EmoteSetKind::Global | EmoteSetKind::Special => {}
	}

	let mut collected_emotes = vec![];

	for (emote_id, e) in emote_set.emotes.into_iter().flatten().filter_map(|e| e.id.map(|id| (id, e))) {
		if !emotes.contains_key(&emote_id.into()) {
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

		collected_emotes.push(EmoteSetEmote {
			id: emote_id.into(),
			alias: emote_name,
			added_at: e.timestamp.map(|t| t.into_chrono()).unwrap_or_default(),
			flags,
			added_by_id: e.actor_id.map(Into::into),
			origin_set_id: None,
		});
	}

	emote_sets.insert(
		emote_set.id.into(),
		EmoteSet {
			id: emote_set.id.into(),
			name: emote_set.name,
			description: None,
			tags: emote_set.tags,
			emotes: collected_emotes,
			capacity: Some(emote_set.capacity),
			owner_id: matches!(kind, EmoteSetKind::Personal | EmoteSetKind::Normal).then_some(emote_set.owner_id.into()),
			origin_config: None,
			kind,
			emotes_changed_since_reindex: true,
			search_updated_at: None,
			updated_at: chrono::Utc::now(),
		},
	);

	outcome
}
