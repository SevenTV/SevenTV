use std::{sync::Arc, time::Duration};

use anyhow::Context;
use async_nats::jetstream::{consumer, stream};
use futures::StreamExt;
use scuffle_foundations::context::{self, ContextFutExt};

use crate::global::Global;

const JETSTREAM_NAME: &str = "image-processor-callback";
const JETSTREAM_CONSUMER_NAME: &str = "image-processor-callback-consumer";

pub async fn run(global: Arc<Global>) -> Result<(), anyhow::Error> {
	let config = &global.config().extra.api.image_processor;

	let subject = if config.event_queue_topic_prefix.is_empty() {
		"emote.*".to_owned()
	} else {
		format!("{}.emote.*", config.event_queue_topic_prefix)
	};

	let stream = global
		.jetstream()
		.get_or_create_stream(stream::Config {
			name: JETSTREAM_NAME.to_string(),
			max_consumers: 1,
			subjects: vec![subject],
			retention: stream::RetentionPolicy::WorkQueue,
			..Default::default()
		})
		.await
        .context("failed to create image processor callback stream")?;

    let consumer = stream
        .get_or_create_consumer(JETSTREAM_CONSUMER_NAME, consumer::pull::Config {
            name: Some(JETSTREAM_CONSUMER_NAME.to_string()),
            ack_policy: consumer::AckPolicy::Explicit,
            ack_wait: Duration::from_secs(30),
            ..Default::default()
        })
        .await
        .context("failed to create image processor callback consumer")?;

    let mut consumer = consumer.messages().await.context("failed to get image processor callback consumer messages")?;

    while let Some(message) = consumer.next().with_context(context::Context::global()).await {
        let message = message.context("consumer closed")?.context("failed to get message")?;
        // decode and handle message
    }

	Ok(())
}
