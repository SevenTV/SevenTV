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
			"api",
			jetstream::consumer::pull::Config {
				name: Some("api".to_string()),
				durable_name: Some("api".to_string()),
				ack_policy: jetstream::consumer::AckPolicy::Explicit,
				filter_subject: format!("{}.response", global.config.cdn.purge_stream_subject),
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

					if let Err(e) = purge_cloudflare(&global, payload).await {
						tracing::error!("error purging keys: {:#}", e);
						msg.ack_with(jetstream::AckKind::Nak(Some(Duration::from_secs(10))))
							.await
							.map_err(|err| anyhow::anyhow!("n ack: {err:#}"))?;
					} else {
						msg.ack().await.map_err(|err| anyhow::anyhow!("ack: {err:#}"))?;
					}
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

#[tracing::instrument(skip_all, fields(keys = ?req.files.iter().map(|k| k.to_string()).collect::<Vec<_>>()))]
async fn purge_cloudflare(global: &Arc<Global>, req: shared::cdn::PurgeRequest) -> anyhow::Result<()> {
	tracing::info!("purging keys from cloudflare");

	let resp = global
		.http_client
		.post(format!(
			"https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
			global.config.cdn.cloudflare_cdn_zone_id
		))
		.header("Authorization", format!("Bearer {}", global.config.cdn.cloudflare_api_token))
		.json(&serde_json::json!({
			"files": req.files.iter().filter_map(|k| {
				global.config.api.cdn_origin.join(&k.to_string()).ok()
			}).collect::<Vec<_>>()
		}))
		.send()
		.await
		.context("cloudflare purge request")?;

	let status = resp.status();

	let body = resp.text().await.context("cloudflare purge request failed")?;

	if !status.is_success() {
		anyhow::bail!("cloudflare purge request failed: {:#}", body);
	}

	tracing::info!("cloudflare purge request successful: {:#}", body);

	Ok(())
}
