use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Context;
use bson::doc;
use futures::StreamExt;
use mongodb::bson::oid::ObjectId;
use shared::database::emote::EmoteId;
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestKind, EmoteModerationRequestStatus,
};
use shared::database::MongoCollection;

use super::ProcessOutcome;
use crate::global::Global;
use crate::jobs::JobOutcome;
use crate::types;

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub mod_requests: &'a mut Vec<EmoteModerationRequest>,
}

pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("messages");

	let RunInput { global, mod_requests } = input;

	let mut read = HashMap::new();

	tracing::info!("loading messages_read collection");
	let mut cursor = global
		.source_db()
		.collection::<types::MessageRead>("messages_read")
		.find(doc! {})
		.await
		.context("query")?;
	while let Some(message) = cursor.next().await {
		match message {
			Ok(message) => {
				read.insert(message.message_id, message.read);
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}
	}

	let mut dedupe_mod_requests = HashSet::new();

	let mut cursor = global
		.source_db()
		.collection::<types::Message>("messages")
		.find(doc! {})
		.await
		.context("query")?;

	while let Some(message) = cursor.next().await {
		match message {
			Ok(message) => {
				outcome += process(ProcessInput {
					read: &mut read,
					dedupe_mod_requests: &mut dedupe_mod_requests,
					mod_requests,
					message,
				});
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}

struct ProcessInput<'a> {
	pub read: &'a mut HashMap<ObjectId, bool>,
	pub dedupe_mod_requests: &'a mut HashSet<(EmoteId, EmoteModerationRequestKind)>,
	pub mod_requests: &'a mut Vec<EmoteModerationRequest>,
	pub message: types::Message,
}

fn process(input: ProcessInput<'_>) -> ProcessOutcome {
	let ProcessInput {
		read,
		dedupe_mod_requests,
		mod_requests,
		message,
	} = input;

	let outcome = ProcessOutcome::default();

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

	let status = match read.get(&message.id) {
		Some(true) => EmoteModerationRequestStatus::Approved,
		_ => EmoteModerationRequestStatus::Pending,
	};

	if dedupe_mod_requests.insert((emote_id, kind)) {
		mod_requests.push(EmoteModerationRequest {
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
	}

	outcome
}
