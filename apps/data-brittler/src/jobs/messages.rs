use std::sync::Arc;

use bson::doc;
use fnv::FnvHashMap;
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::options::InsertManyOptions;
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestKind, EmoteModerationRequestStatus,
};
use shared::database::Collection;

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub struct MessagesJob {
	global: Arc<Global>,
	read: FnvHashMap<ObjectId, bool>,
	mod_requests: Vec<EmoteModerationRequest>,
}

impl Job for MessagesJob {
	type T = types::Message;

	const NAME: &'static str = "transfer_messages";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
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
			mod_requests: vec![],
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("messages")
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

		self.mod_requests.push(EmoteModerationRequest {
			id,
			user_id: message.author_id.into(),
			kind,
			reason: None,
			emote_id,
			status,
			country_code,
			assigned_to: vec![],
			priority: 0,
		});

		if self.mod_requests.len() > 50_000 {
			let Ok(res) = EmoteModerationRequest::collection(self.global.target_db())
				.insert_many(&self.mod_requests)
				.with_options(InsertManyOptions::builder().ordered(false).build())
				.await
			else {
				outcome.errors.push(error::Error::InsertMany);
				return outcome;
			};

			outcome.inserted_rows += res.inserted_ids.len() as u64;
			if res.inserted_ids.len() != self.mod_requests.len() {
				outcome.errors.push(error::Error::InsertMany);
			}

			self.mod_requests.clear();
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
