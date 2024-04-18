use std::pin::Pin;
use std::sync::Arc;

use postgres_types::Type;
use shared::database;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types;

pub struct BansJob {
	global: Arc<Global>,
	bans_writer: Pin<Box<BinaryCopyInWriter>>,
}

impl Job for BansJob {
	type T = types::Ban;

	const NAME: &'static str = "transfer_bans";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating user_bans table");
			scuffle_utils::database::query("TRUNCATE user_bans")
				.build()
				.execute(global.source_db())
				.await?;
		}

		let bans_client = global.source_db().get().await?;
		let bans_writer = BinaryCopyInWriter::new(
			bans_client
				.copy_in("COPY user_bans (id, user_id, created_by_id, data, expires_at) FROM STDIN WITH (FORMAT BINARY)")
				.await?,
			&[Type::UUID, Type::UUID, Type::UUID, Type::JSONB, Type::TIMESTAMPTZ],
		);

		Ok(Self {
			global,
			bans_writer: Box::pin(bans_writer),
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().database("7tv").collection("bans")
	}

	async fn process(&mut self, ban: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		// TODO: effects
		let data = database::UserBanData { reason: ban.reason };

		match self
			.bans_writer
			.as_mut()
			.write(&[
				&ban.id.into_ulid(),
				&ban.victim_id.into_ulid(),
				&ban.actor_id.into_ulid(),
				&postgres_types::Json(data),
				&ban.expire_at.into_chrono(),
			])
			.await
		{
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}

	async fn finish(mut self) -> anyhow::Result<()> {
		self.bans_writer.as_mut().finish().await?;
		Ok(())
	}
}
