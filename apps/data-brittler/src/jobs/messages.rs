use std::mem;
use std::sync::Arc;

use bson::doc;
use fnv::{FnvHashMap, FnvHashSet};
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::options::InsertManyOptions;
use shared::database::emote::EmoteId;
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestKind, EmoteModerationRequestStatus,
};
use shared::database::MongoCollection;

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

const BATCH_SIZE: usize = 50_000;

pub struct MessagesJob {
	global: Arc<Global>,
	read: FnvHashMap<ObjectId, bool>,
	dedupe_mod_requests: FnvHashSet<(EmoteId, EmoteModerationRequestKind)>,
	mod_requests: Vec<EmoteModerationRequest>,
}

impl Job for MessagesJob {
	type T = types::Message;

	const NAME: &'static str = "transfer_messages";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		EmoteModerationRequest::collection(global.target_db()).drop().await?;
		let indexes = EmoteModerationRequest::indexes();
		if !indexes.is_empty() {
			EmoteModerationRequest::collection(global.target_db())
				.create_indexes(indexes)
				.await?;
		}

		let mut read = FnvHashMap::default();

		tracing::info!("loading messages_read collection");
		let mut cursor = global
			.source_db()
			.collection::<types::MessageRead>("messages_read")
			.find(doc! {})
			.await?;
		while let Some(message) = cursor.try_next().await? {
			read.insert(message.message_id, message.read);
		}

		Ok(Self {
			global,
			read,
			dedupe_mod_requests: FnvHashSet::default(),
			mod_requests: Vec::new(),
		})
	}

	async fn collection(&self) -> Option<mongodb::Collection<Self::T>> {
		Some(self.global.source_db().collection("messages"))
	}

	async fn process(&mut self, message: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let id = message.id.into();

		let (kind, emote_id, country_code) = match message.data {
			types::MessageData::EmoteRequest {
				target_id,
				wish: Some(types::EmoteWish::List),
				actor_country_code,
			} => (
				EmoteModerationRequestKind::PublicListing,
				target_id.into(),
				actor_country_code,
			),
			types::MessageData::EmoteRequest {
				target_id,
				wish: Some(types::EmoteWish::PersonalUse),
				actor_country_code,
			} => (EmoteModerationRequestKind::PersonalUse, target_id.into(), actor_country_code),
			// inbox messages are not tickets
			_ => return outcome,
		};

		let status = match self.read.get(&message.id) {
			Some(true) => EmoteModerationRequestStatus::Approved,
			_ => EmoteModerationRequestStatus::Pending,
		};

		if self.dedupe_mod_requests.insert((emote_id, kind)) {
			self.mod_requests.push(EmoteModerationRequest {
				id,
				user_id: message.author_id.into(),
				kind,
				reason: None,
				emote_id,
				priority: if status == EmoteModerationRequestStatus::Pending {
					100
				} else {
					0
				},
				status,
				country_code,
				assigned_to: vec![],
				search_updated_at: None,
				updated_at: chrono::Utc::now(),
			});
		} else {
			// Happens too often, so we just ignore it

			// outcome.errors.push(error::Error::DuplicateEmoteModRequest {
			// 	emote_id: mod_request.emote_id,
			// 	kind: mod_request.kind,
			// });
		}

		if self.mod_requests.len() >= BATCH_SIZE {
			match EmoteModerationRequest::collection(self.global.target_db())
				.insert_many(mem::take(&mut self.mod_requests))
				.await
			{
				Ok(res) => {
					outcome.inserted_rows += res.inserted_ids.len() as u64;
					if res.inserted_ids.len() < BATCH_SIZE {
						return outcome.with_error(error::Error::InsertMany);
					}
				}
				Err(e) => return outcome.with_error(e),
			}
		}

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing messages job");

		let mut outcome = ProcessOutcome::default();

		match EmoteModerationRequest::collection(self.global.target_db())
			.insert_many(&self.mod_requests)
			.with_options(InsertManyOptions::builder().ordered(false).build())
			.await
		{
			Ok(res) => {
				outcome.inserted_rows += res.inserted_ids.len() as u64;
				if res.inserted_ids.len() != self.mod_requests.len() {
					outcome.errors.push(error::Error::InsertMany);
				}
			}
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}
}
