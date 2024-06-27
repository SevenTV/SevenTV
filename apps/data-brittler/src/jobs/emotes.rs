use std::sync::Arc;

use mongodb::bson::doc;
use mongodb::options::InsertManyOptions;
use shared::database::emote::{Emote, EmoteFlags, EmoteMerged};
use shared::database::image_set::{ImageSet, ImageSetInput};
use shared::database::user::UserId;
use shared::database::Collection;
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
			Emote::collection(global.target_db()).delete_many(doc! {}, None).await?;
		}

		Ok(Self { global, emotes: vec![] })
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("emotes")
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

			let image_set = ImageSet {
				input: ImageSetInput::Image(v.input_file.into()),
				outputs: v.image_files.into_iter().map(Into::into).collect(),
			};

			self.emotes.push(Emote {
				id: v.id.into(),
				owner_id: (!owner_id.is_nil() && !owner_id.is_one())
					.then_some(owner_id)
					.unwrap_or(UserId::nil()),
				default_name: v.name.unwrap_or_else(|| emote.name.clone()),
				tags: emote.tags.clone(),
				animated: v.animated,
				image_set,
				flags,
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
