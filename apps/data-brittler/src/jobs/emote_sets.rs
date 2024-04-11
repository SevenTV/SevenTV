use std::pin::Pin;
use std::sync::Arc;

use chrono::Utc;
use postgres_types::{Json, Type};
use shared::database::{EmoteSetEmoteFlag, EmoteSetKind, EmoteSetSettings};
use tokio_postgres::binary_copy::BinaryCopyInWriter;

use super::{Job, ProcessOutcome};
use crate::database::emote_set_kind_type;
use crate::global::Global;
use crate::types;

pub struct EmoteSetsJob {
	global: Arc<Global>,
	emote_sets_writer: Pin<Box<BinaryCopyInWriter>>,
	emote_set_emotes_writer: Pin<Box<BinaryCopyInWriter>>,
}

impl Job for EmoteSetsJob {
	type T = types::EmoteSet;

	const NAME: &'static str = "transfer_emote_sets";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating emote_sets and emote_set_emotes table");
			scuffle_utils::database::query("TRUNCATE emote_sets, emote_set_emotes")
				.build()
				.execute(global.db())
				.await?;
		}

		let emote_sets_client = global.db().get().await?;
		let emote_sets_writer = BinaryCopyInWriter::new(
			emote_sets_client
				.copy_in("COPY emote_sets (id, owner_id, name, kind, tags, settings) FROM STDIN WITH (FORMAT BINARY)")
				.await?,
			&[
				Type::UUID,
				Type::UUID,
				Type::VARCHAR,
				emote_set_kind_type(&global).await?,
				Type::TEXT_ARRAY,
				Type::JSONB,
			],
		);

		let emote_set_emotes_client = global.db().get().await?;
		let emote_set_emotes_writer = BinaryCopyInWriter::new(
			emote_set_emotes_client
				.copy_in(
					"COPY emote_set_emotes (emote_set_id, emote_id, added_by_id, name, flags, added_at) FROM STDIN WITH (FORMAT BINARY)",
				)
				.await?,
			&[
				Type::UUID,
				Type::UUID,
				Type::UUID,
				Type::VARCHAR,
				Type::INT4,
				Type::TIMESTAMPTZ,
			],
		);

		Ok(Self {
			global,
			emote_sets_writer: Box::pin(emote_sets_writer),
			emote_set_emotes_writer: Box::pin(emote_set_emotes_writer),
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.mongo().database("7tv").collection("emote_sets")
	}

	async fn process(&mut self, emote_set: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let id = emote_set.id.into_ulid();

		let kind = if emote_set.flags.contains(types::EmoteSetFlagModel::Personal) {
			EmoteSetKind::Personal
		} else {
			EmoteSetKind::Normal
		};

		let settings = EmoteSetSettings {
			capacity: emote_set.capacity,
			privileged: emote_set.privileged || emote_set.flags.contains(types::EmoteSetFlagModel::Privileged),
			immutable: emote_set.immutable || emote_set.flags.contains(types::EmoteSetFlagModel::Immutable),
		};

		match self
			.emote_sets_writer
			.as_mut()
			.write(&[
				&id,
				&emote_set.owner_id.into_ulid(),
				&emote_set.name,
				&kind,
				&emote_set.tags,
				&Json(settings),
			])
			.await
		{
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => {
				outcome.errors.push(e.into());
				return outcome;
			}
		}

		for (emote_id, e) in emote_set
			.emotes
			.into_iter()
			.filter_map(|e| e)
			.filter_map(|e| e.id.map(|id| (id, e)))
		{
			let emote_id = emote_id.into_ulid();

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

			match self
				.emote_set_emotes_writer
				.as_mut()
				.write(&[
					&id,
					&emote_id,
					&e.actor_id.map(|a| a.into_ulid()),
					&e.name,
					&flags.bits(),
					&e.timestamp.map(|t| t.into_chrono()).unwrap_or(Utc::now()),
				])
				.await
			{
				Ok(_) => outcome.inserted_rows += 1,
				Err(e) => outcome.errors.push(e.into()),
			}
		}

		outcome
	}

	async fn finish(mut self) -> anyhow::Result<()> {
		self.emote_sets_writer.as_mut().finish().await?;
		self.emote_set_emotes_writer.as_mut().finish().await?;

		Ok(())
	}
}
