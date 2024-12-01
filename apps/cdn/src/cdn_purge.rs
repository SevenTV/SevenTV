use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_nats::jetstream;
use async_nats::jetstream::stream::RetentionPolicy;
use futures::StreamExt;
use scuffle_context::{ContextFutExt, ContextStreamExt};

use crate::global::Global;

pub async fn run(global: Arc<Global>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
	let stream = global
		.jetstream
		.get_or_create_stream(async_nats::jetstream::stream::Config {
			name: global.config.cdn.purge_stream_name.clone(),
			subjects: vec![format!("{}.>", global.config.cdn.purge_stream_subject)],
			retention: RetentionPolicy::Interest,
			max_age: Duration::from_secs(60 * 60 * 24),
			..Default::default()
		})
		.await
		.context("jetstream")?;

	let consumer = stream
		.get_or_create_consumer(
			&global.config.pod.name,
			jetstream::consumer::pull::Config {
				name: Some(global.config.pod.name.clone()),
				ack_policy: jetstream::consumer::AckPolicy::All,
				filter_subject: format!("{}.request", global.config.cdn.purge_stream_subject),
				inactive_threshold: Duration::from_secs(60 * 60 * 24),
				max_deliver: 10,
				..Default::default()
			},
		)
		.await
		.context("consumer")?;

	tracing::info!("cdn purge worker started");

	while !ctx.is_done() {
		let messages = consumer.messages().await.context("consumer")?.with_context(&ctx);
		let mut messages = std::pin::pin!(messages);

		while let Some(msg) = messages.next().await {
			match msg {
				Ok(msg) => {
					let payload = match serde_json::from_slice::<shared::cdn::PurgeRequest>(&msg.payload) {
						Ok(payload) => payload,
						Err(e) => {
							tracing::error!("error parsing payload: {:#}", e);
							continue;
						}
					};

					tracing::info!(files = %payload.files.len(), "purging keys");

					for file in payload.files {
						global.cache.purge(file).await;
					}

					global
						.jetstream
						.publish(
							format!("{}.response", global.config.cdn.purge_stream_subject),
							msg.payload.clone(),
						)
						.await
						.context("publish")?;

					msg.ack().await.map_err(|err| anyhow::anyhow!("ack: {err:#}"))?;
				}
				Err(e) => {
					tracing::error!("error receiving message: {:#}", e);
				}
			}
		}

		if ctx.is_done() {
			break;
		}

		tracing::info!("message stream closed, waiting 10 seconds before reconnecting");
		tokio::time::sleep(Duration::from_secs(10)).with_context(&ctx).await;
	}

	Ok(())
}
