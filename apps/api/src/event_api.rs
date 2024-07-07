use std::{
	hash::{DefaultHasher, Hash, Hasher},
	iter,
	sync::atomic::{AtomicU64, Ordering},
};

use sha2::Digest;
use shared::event_api::{
	payload::Dispatch,
	types::{ChangeMap, EventType},
	Message,
};

pub struct EventApi {
	nats: async_nats::Client,
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
	pub fn new(nats: async_nats::Client) -> Self {
		Self {
			nats,
			sequence: AtomicU64::new(0),
		}
	}

	pub async fn dispatch_event(
		&self,
		ty: EventType,
		body: ChangeMap,
		condition: Option<(&'static str, String)>,
	) -> Result<(), EventApiError> {
		let mut nats_subject = vec![];
		nats_subject.push("events.op.dispatch.type".to_string());
		nats_subject.push(ty.to_string());

		if let Some((k, v)) = &condition {
			let mut hasher = sha2::Sha256::new();

			hasher.update(k);
			hasher.update(v);

            let cond_hash = hex::encode(hasher.finalize());

			nats_subject.push(cond_hash);
		}

		let nats_subject = nats_subject.join(".");

		let condition = condition
			.map(|c| vec![iter::once((c.0.to_string(), c.1)).collect()])
			.unwrap_or_default();

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
				condition,
				whisper: None,
			},
			self.sequence.fetch_add(1, Ordering::SeqCst),
		);

		self.nats
			.publish(nats_subject, serde_json::to_string(&message)?.into_bytes().into())
			.await?;

		Ok(())
	}
}
