use std::sync::Arc;

use mongodb::options::InsertManyOptions;
use shared::database::emote::{Emote, EmoteFlags, EmoteMerged};
use shared::database::image_set::{self, ImageSet, ImageSetInput};
use shared::database::user::UserId;
use shared::database::MongoCollection;
use shared::old_types::EmoteFlagsModel;

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types::EmoteLifecycle;
use crate::{error, types};

pub struct EmotesJob {
	global: Arc<Global>,
	emotes: Vec<Emote>,
}

impl Job for EmotesJob {
	type T = types::Emote;

	const NAME: &'static str = "transfer_emotes";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping emotes collection");
			Emote::collection(global.target_db()).drop().await?;
			let indexes = Emote::indexes();
			if !indexes.is_empty() {
				Emote::collection(global.target_db()).create_indexes(indexes).await?;
			}
		}

		Ok(Self { global, emotes: vec![] })
	}

	async fn collection(&self) -> Option<mongodb::Collection<Self::T>> {
		Some(self.global.source_db().collection("emotes"))
	}

	async fn process(&mut self, emote: Self::T) -> ProcessOutcome {
		let owner_id = UserId::from(emote.owner_id);

		for v in emote.versions {
			if (v.state.lifecycle == EmoteLifecycle::Failed)
				|| (v.state.lifecycle == EmoteLifecycle::Deleted && v.state.replace_id.is_none())
			{
				continue;
			}

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

			self.emotes.push(Emote {
				id: v.id.into(),
				owner_id: if !owner_id.is_nil() && !owner_id.is_one() {
					owner_id
				} else {
					UserId::nil()
				},
				default_name: v.name.unwrap_or_else(|| emote.name.clone()),
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
			});
		}

		ProcessOutcome::default()
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing emotes job");

		let mut outcome = ProcessOutcome::default();

		let res = Emote::collection(self.global.target_db())
			.insert_many(&self.emotes)
			.with_options(InsertManyOptions::builder().ordered(false).build())
			.await;

		match res {
			Ok(res) => {
				outcome.inserted_rows += res.inserted_ids.len() as u64;
				if res.inserted_ids.len() != self.emotes.len() {
					outcome.errors.push(error::Error::InsertMany);
				}
			}
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}
}
