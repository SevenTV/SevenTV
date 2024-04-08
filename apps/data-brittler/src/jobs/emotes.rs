use std::pin::Pin;
use std::sync::Arc;

use postgres_types::Type;
use shared::database::{EmoteSettings, FileSetKind, FileSetProperties};
use shared::types::old::EmoteFlagsModel;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

use super::{Job, ProcessOutcome};
use crate::database::file_set_kind_type;
use crate::global::Global;
use crate::types::image_files_to_file_properties;
use crate::{error, types};

pub struct EmotesJob {
	global: Arc<Global>,
	file_sets_writer: Pin<Box<BinaryCopyInWriter>>,
	emotes_writer: Pin<Box<BinaryCopyInWriter>>,
}

impl EmotesJob {
	pub async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating emotes table");
			scuffle_utils::database::query("TRUNCATE emotes")
				.build()
				.execute(global.db())
				.await?;

			tracing::info!("deleting emotes files from file_sets table");
			scuffle_utils::database::query("DELETE FROM file_sets WHERE kind = 'EMOTE'")
				.build()
				.execute(global.db())
				.await?;
		}

		let file_sets_client = global.db().get().await?;
		let file_sets_writer = BinaryCopyInWriter::new(
			file_sets_client
				.copy_in("COPY file_sets (id, kind, authenticated, properties) FROM STDIN WITH (FORMAT BINARY)")
				.await?,
			&[Type::UUID, file_set_kind_type(&global).await?, Type::BOOL, Type::JSONB],
		);

		let emotes_client = global.db().get().await?;
		let emotes_writer = BinaryCopyInWriter::new(
			emotes_client
				.copy_in(
					"COPY emotes (id, owner_id, default_name, tags, animated, settings, file_set_id) FROM STDIN WITH (FORMAT BINARY)",
				)
				.await?,
			&[
				Type::UUID,
				Type::UUID,
				Type::VARCHAR,
				Type::TEXT_ARRAY,
				Type::BOOL,
				Type::JSONB,
				Type::UUID,
			],
		);

		Ok(Self {
			global,
			file_sets_writer: Box::pin(file_sets_writer),
			emotes_writer: Box::pin(emotes_writer),
		})
	}
}

impl Job for EmotesJob {
	type T = types::Emote;

	const NAME: &'static str = "transfer_emotes";

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.mongo().database("7tv").collection("emotes")
	}

	async fn process(&mut self, emote: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		for v in emote.versions {
			let created_at = v.created_at.into_chrono().into();

			let file_set_id = ulid::Ulid::from_datetime(created_at);

			let outputs = match image_files_to_file_properties(v.image_files) {
				Ok(outputs) => outputs,
				Err(e) => {
					outcome.errors.push(e.into());
					continue;
				}
			};

			match self
				.file_sets_writer
				.as_mut()
				.write(&[
					&file_set_id,
					&FileSetKind::Emote,
					&false,
					&postgres_types::Json(FileSetProperties::Image {
						input: v.input_file.into(),
						pending: false,
						outputs,
					}),
				])
				.await
			{
				Ok(_) => outcome.inserted_rows += 1,
				Err(e) => {
					outcome.errors.push(e.into());
					continue;
				}
			}

			match self
				.emotes_writer
				.as_mut()
				.write(&[
					&ulid::Ulid::from_datetime(created_at),
					&emote.owner_id.into_ulid(),
					&v.name.unwrap_or_else(|| emote.name.clone()),
					&emote.tags,
					&v.animated,
					&postgres_types::Json(EmoteSettings {
						public_listed: v.state.listed,
						default_zero_width: emote.flags.contains(EmoteFlagsModel::ZeroWidth),
						approved_personal: Some(v.state.allow_personal),
						nsfw: emote.flags.contains(EmoteFlagsModel::Sexual),
						private: emote.flags.contains(EmoteFlagsModel::Private),
					}),
					&file_set_id,
				])
				.await
			{
				Ok(_) => outcome.inserted_rows += 1,
				Err(e) => outcome.errors.push(error::Error::Db(e)),
			}
		}

		outcome
	}

	async fn finish(self) -> anyhow::Result<()> {
		tracing::info!("finishing emotes job");

		// self.file_sets_writer.into_inner().await?.as_mut().close().await?;
		tracing::info!("finished writing emote file sets");

		// self.emotes_writer.into_inner().await?.as_mut().close().await?;
		tracing::info!("finished writing emotes");

		Ok(())
	}
}
