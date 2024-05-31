use std::sync::Arc;

use mongodb::options::InsertManyOptions;
use shared::database::{Collection, Emote, EmoteFlags, ImageSet, ImageSetInput};
use shared::old_types::EmoteFlagsModel;

use super::{Job, ProcessOutcome};
use crate::global::Global;
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
			Emote::collection(global.target_db()).drop(None).await?;
		}

		Ok(Self { global, emotes: vec![] })
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("emotes")
	}

	async fn process(&mut self, emote: Self::T) -> ProcessOutcome {
		for v in emote.versions {
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

			let image_set = ImageSet {
				input: ImageSetInput::Image(v.input_file.into()),
				outputs: v.image_files.into_iter().map(Into::into).collect(),
			};

			self.emotes.push(Emote {
				id: v.id.into(),
				owner_id: Some(emote.owner_id.into()),
				default_name: v.name.unwrap_or_else(|| emote.name.clone()),
				tags: emote.tags.clone(),
				animated: v.animated,
				image_set,
				flags,
				attribution: vec![],
			});
		}

		ProcessOutcome::default()
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing emotes job");

		let mut outcome = ProcessOutcome::default();

		let res = Emote::collection(self.global.target_db())
			.insert_many(&self.emotes, InsertManyOptions::builder().ordered(false).build())
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
