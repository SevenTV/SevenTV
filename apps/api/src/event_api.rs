use std::hash::{DefaultHasher, Hash, Hasher};
use std::iter;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::Digest;
use shared::database::Id;
use shared::event_api::payload::Dispatch;
use shared::event_api::types::{ChangeMap, EventType};
use shared::event_api::Message;

pub struct EventApi {
	nats: async_nats::Client,
	prefix: String,
	sequence: AtomicU64,
}

#[derive(thiserror::Error, Debug)]
pub enum EventApiError {
	#[error("publish error: {0}")]
	PublishError(#[from] async_nats::PublishError),
	#[error("serialize error: {0}")]
	Serialize(#[from] serde_json::Error),
}

impl EventApi {
	pub fn new(nats: async_nats::Client, prefix: &str) -> Self {
		Self {
			nats,
			sequence: AtomicU64::new(0),
			prefix: prefix.to_string(),
		}
	}

	pub async fn dispatch_event<S>(
		&self,
		ty: EventType,
		body: ChangeMap,
		condition_object_id: Id<S>,
	) -> Result<(), EventApiError> {
		let mut nats_subject = vec![];
		nats_subject.push(self.prefix.clone());
		nats_subject.push("events.op.dispatch.type".to_string());
		nats_subject.push(ty.to_string());

		let mut hasher = sha2::Sha256::new();

		hasher.update("object_id");
		hasher.update(condition_object_id.to_string());

		let cond_hash = hex::encode(hasher.finalize());

		nats_subject.push(cond_hash);

		let nats_subject = nats_subject.join(".");

		let mut hasher = DefaultHasher::new();
		hasher.write(&body.id.into_bytes());
		hasher.write(body.kind.as_str().as_bytes());
		body.object.hash(&mut hasher);
		let hash = hasher.finish();

		let message = Message::new(
			Dispatch {
				ty,
				body,
				hash: Some(hash as u32),
				effect: None,
				matches: vec![],
				condition: vec![iter::once(("object_id".to_string(), condition_object_id.to_string())).collect()],
				whisper: None,
			},
			self.sequence.fetch_add(1, Ordering::SeqCst),
		);

		tracing::debug!(subject = %nats_subject, message = ?message, "dispatching event");

		self.nats
			.publish(nats_subject, serde_json::to_string(&message)?.into_bytes().into())
			.await?;

		Ok(())
	}
}
