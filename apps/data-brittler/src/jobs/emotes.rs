use std::sync::Arc;

use mongodb::bson::doc;
use shared::database::{Collection, Emote, EmoteFlags, FileSet, FileSetKind, FileSetProperties};
use shared::types::old::EmoteFlagsModel;

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types::image_files_to_file_properties;
use crate::{error, types};

pub struct EmotesJob {
	global: Arc<Global>,
}

impl Job for EmotesJob {
	type T = types::Emote;

	const NAME: &'static str = "transfer_emotes";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating emotes collection");
			Emote::collection(global.target_db()).delete_many(doc! {}, None).await?;

			tracing::info!("deleting emotes files from file_sets collection");
			FileSet::collection(global.target_db())
				.delete_many(doc! { "kind": mongodb::bson::to_bson(&FileSetKind::Emote)? }, None)
				.await?;
		}

		Ok(Self { global })
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("emotes")
	}

	async fn process(&mut self, emote: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		for v in emote.versions {
			let created_at = v.created_at.into_chrono().into();

			let file_set_id = crate::database::object_id_from_datetime(created_at);

			let outputs = match image_files_to_file_properties(v.image_files) {
				Ok(outputs) => outputs,
				Err(e) => {
					outcome.errors.push(e.into());
					continue;
				}
			};

			match FileSet::collection(self.global.target_db())
				.insert_one(
					FileSet {
						id: file_set_id,
						kind: FileSetKind::Emote,
						authenticated: false,
						properties: FileSetProperties::Image {
							input: v.input_file.into(),
							pending: false,
							outputs,
						},
					},
					None,
				)
				.await
			{
				Ok(_) => outcome.inserted_rows += 1,
				Err(e) => {
					outcome.errors.push(e.into());
					continue;
				}
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

			match Emote::collection(self.global.target_db())
				.insert_one(
					Emote {
						id: crate::database::object_id_from_datetime(created_at),
						owner_id: Some(emote.owner_id),
						default_name: v.name.unwrap_or_else(|| emote.name.clone()),
						tags: emote.tags.clone(),
						animated: v.animated,
						file_set_id,
						flags,
						attribution: vec![],
					},
					None,
				)
				.await
			{
				Ok(_) => outcome.inserted_rows += 1,
				Err(e) => outcome.errors.push(error::Error::Db(e)),
			}
		}

		outcome
	}
}
