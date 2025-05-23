use fred::interfaces::{ClientLike, PubsubInterface};
use fred::types::config::Config;
use image_processor_proto::EventCallback;
use prost::Message;

use super::{EventQueue, EventQueueError};
use crate::config::{MessageEncoding, RedisEventQueueConfig};

#[derive(Debug)]
pub struct RedisEventQueue {
	client: fred::clients::Client,
	name: String,
	message_encoding: MessageEncoding,
}

#[derive(Debug, thiserror::Error)]
pub enum RedisEventQueueError {
	#[error("redis: {0}")]
	Redis(#[from] fred::error::Error),
	#[error("json encode: {0}")]
	JsonEncode(#[from] serde_json::Error),
}

impl RedisEventQueue {
	#[tracing::instrument(skip(config), name = "RedisEventQueue::new", fields(name = %config.name), err)]
	pub async fn new(config: &RedisEventQueueConfig) -> Result<Self, EventQueueError> {
		Ok(Self {
			client: fred::clients::Client::new(
				Config::from_url(&config.url).map_err(RedisEventQueueError::from)?,
				None,
				None,
				None,
			),
			name: config.name.clone(),
			message_encoding: config.message_encoding,
		})
	}
}

impl EventQueue for RedisEventQueue {
	fn name(&self) -> &str {
		&self.name
	}

	#[tracing::instrument(skip(self), name = "RedisEventQueue::publish", err)]
	async fn publish(&self, topic: &str, data: EventCallback) -> Result<(), EventQueueError> {
		let payload = if self.message_encoding == MessageEncoding::Protobuf {
			data.encode_to_vec()
		} else {
			serde_json::to_string(&data)
				.map_err(RedisEventQueueError::JsonEncode)?
				.into_bytes()
		};

		self.client
			.publish::<(), _, _>(topic, payload)
			.await
			.map_err(RedisEventQueueError::Redis)?;

		Ok(())
	}

	async fn healthy(&self) -> bool {
		self.client.ping::<()>(None).await.is_ok()
	}
}
