use std::sync::Arc;

use mongodb::bson::doc;
use mongodb::options::InsertManyOptions;
use shared::database::{Collection, Emote, EmoteFlags, FileSet, FileSetId, FileSetKind, FileSetProperties};
use shared::types::old::EmoteFlagsModel;

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types::image_files_to_file_properties;
use crate::{error, types};

pub struct EmotesJob {
	global: Arc<Global>,
	emotes: Vec<Emote>,
	file_sets: Vec<FileSet>,
}

impl Job for EmotesJob {
	type T = types::Emote;

	const NAME: &'static str = "transfer_emotes";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping emotes collection");
			Emote::collection(global.target_db()).drop(None).await?;

			tracing::info!("deleting emotes files from file_sets collection");
			FileSet::collection(global.target_db())
				.delete_many(doc! { "kind": mongodb::bson::to_bson(&FileSetKind::Emote)? }, None)
				.await?;
		}

		Ok(Self {
			global,
			emotes: vec![],
			file_sets: vec![],
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("emotes")
	}

	async fn process(&mut self, emote: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		for v in emote.versions {
			let file_set_id = FileSetId::with_timestamp(v.created_at.into_chrono());

			let outputs = match image_files_to_file_properties(v.image_files) {
				Ok(outputs) => outputs,
				Err(e) => {
					outcome.errors.push(e.into());
					continue;
				}
			};

			self.file_sets.push(FileSet {
				id: file_set_id,
				kind: FileSetKind::Emote,
				authenticated: false,
				properties: FileSetProperties::Image {
					input: v.input_file.into(),
					pending: false,
					outputs,
				},
			});

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

			self.emotes.push(Emote {
				id: v.id.into(),
				owner_id: Some(emote.owner_id.into()),
				default_name: v.name.unwrap_or_else(|| emote.name.clone()),
				tags: emote.tags.clone(),
				animated: v.animated,
				file_set_id,
				flags,
				attribution: vec![],
			});
		}

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing emotes job");

		let mut outcome = ProcessOutcome::default();

		let insert_options = InsertManyOptions::builder().ordered(false).build();
		let emotes = Emote::collection(self.global.target_db());
		let file_sets = FileSet::collection(self.global.target_db());

		let res = tokio::join!(
			emotes.insert_many(&self.emotes, insert_options.clone()),
			file_sets.insert_many(&self.file_sets, insert_options.clone()),
		);
		let res = vec![res.0, res.1]
			.into_iter()
			.zip(vec![self.emotes.len(), self.file_sets.len()]);

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
