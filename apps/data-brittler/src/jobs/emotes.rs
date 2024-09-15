use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use shared::database::emote::{Emote, EmoteFlags, EmoteId, EmoteMerged};
use shared::database::image_set::{self, ImageSet, ImageSetInput};
use shared::database::queries::filter;
use shared::database::user::UserId;
use shared::database::MongoCollection;
use shared::old_types::EmoteFlagsModel;

use super::{JobOutcome, ProcessOutcome};
use crate::global::Global;
use crate::types::EmoteLifecycle;
use crate::{error, types};

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub emotes: &'a mut HashMap<EmoteId, Emote>,
}

pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("emotes");

	let RunInput { global, emotes } = input;

	let mut cursor = global
		.source_db()
		.collection::<types::Emote>("emotes")
		.find(bson::doc! {})
		.await
		.context("query")?;

	while let Some(emote) = cursor.next().await {
		match emote {
			Ok(emote) => {
				outcome += process(ProcessInput { emotes, emote });
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}

struct ProcessInput<'a> {
	emotes: &'a mut HashMap<EmoteId, Emote>,
	emote: types::Emote,
}

fn process(input: ProcessInput<'_>) -> ProcessOutcome {
	let ProcessInput { emotes, emote } = input;

	let owner_id = UserId::from(emote.owner_id);

	let versions_num = emote.versions.len();

	for v in emote.versions {
		if (v.state.lifecycle == EmoteLifecycle::Failed)
			|| (v.state.lifecycle == EmoteLifecycle::Deleted && v.state.replace_id.is_none())
		{
			continue;
		}

		let default_name = if versions_num == 1 {
			emote.name.clone()
		} else {
			v.name.unwrap_or_else(|| emote.name.clone())
		};

		let mut flags = EmoteFlags::none();
		if emote.flags.contains(EmoteFlagsModel::Private) {
			flags |= EmoteFlags::Private;
		}
		if emote.flags.contains(EmoteFlagsModel::ZeroWidth) {
			flags |= EmoteFlags::DefaultZeroWidth;
		}
		if emote.flags.contains(EmoteFlagsModel::Sexual) {
			flags |= EmoteFlags::Nsfw;
		}
		if v.state.listed {
			flags |= EmoteFlags::PublicListed;
		}
		if v.state.allow_personal {
			flags |= EmoteFlags::ApprovedPersonal;
		}

		if v.animated {
			flags |= EmoteFlags::Animated;
		}

		let aspect_ratio = v.input_file.width as f64 / v.input_file.height as f64;

		let input_file = match image_set::Image::try_from(v.input_file) {
			Ok(input_file) => input_file,
			Err(e) => {
				return ProcessOutcome::default().with_error(error::Error::InvalidCdnFile(e));
			}
		};

		let outputs = match v.image_files.into_iter().map(image_set::Image::try_from).collect() {
			Ok(outputs) => outputs,
			Err(e) => {
				return ProcessOutcome::default().with_error(error::Error::InvalidCdnFile(e));
			}
		};

		let image_set = ImageSet {
			input: ImageSetInput::Image(input_file),
			outputs,
		};

		emotes.insert(
			v.id.into(),
			Emote {
				id: v.id.into(),
				owner_id: if !owner_id.is_nil() && !owner_id.is_one() {
					owner_id
				} else {
					UserId::nil()
				},
				default_name,
				tags: emote.tags.clone(),
				aspect_ratio,
				image_set,
				flags,
				scores: Default::default(),
				search_updated_at: None,
				updated_at: chrono::Utc::now(),
				attribution: vec![],
				merged: v.state.replace_id.map(|id| EmoteMerged {
					target_id: id.into(),
					at: chrono::Utc::now(),
				}),
			},
		);
	}

	ProcessOutcome::default()
}

pub async fn skip(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("emotes");

	let RunInput { global, emotes } = input;

	let mut cursor = Emote::collection(global.target_db())
		.find(filter::filter! {
			Emote {}
		})
		.await
		.context("query")?;

	while let Some(emote) = cursor.next().await {
		match emote {
			Ok(emote) => {
				emotes.insert(emote.id.into(), emote);
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}
