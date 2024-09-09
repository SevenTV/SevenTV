use std::sync::Arc;

use shared::database::product::special_event;
use shared::database::MongoCollection;

use super::{Job, ProcessOutcome};
use crate::global::Global;

pub struct SpecialEventsJob {
	global: Arc<Global>,
}

impl Job for SpecialEventsJob {
	type T = special_event::SpecialEvent;

	const NAME: &'static str = "special_events";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping special_events collections");
			special_event::SpecialEvent::collection(global.target_db())
				.untyped()
				.drop()
				.await?;
			let indexes = special_event::SpecialEvent::indexes();
			if !indexes.is_empty() {
				special_event::SpecialEvent::collection(global.target_db())
					.untyped()
					.create_indexes(indexes)
					.await?;
			}
		}

		Ok(Self { global })
	}

	async fn finish(self) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		if let Err(err) = special_event::SpecialEvent::collection(self.global.target_db())
			.insert_many(
				super::subscriptions::benefits::special_events()
					.into_iter()
					.map(|s| s.special_event),
			)
			.await
		{
			outcome = outcome.with_error(err);
		}

		outcome
	}
}
