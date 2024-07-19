use std::{os::unix::process, sync::Arc};

use anyhow::Context as _;
use async_nats::jetstream::stream::{RetentionPolicy, StorageType};
use futures::StreamExt;
use mongodb::change_stream::event::ChangeStreamEvent;
use scuffle_foundations::context::{Context, ContextFutExt};
use shared::nats::ChangeStreamSubject;

use crate::global::Global;

pub async fn start(global: Arc<Global>) -> anyhow::Result<()> {
    let subject = ChangeStreamSubject::new(&global.config.nats_prefix);

    global.jetstream.get_or_create_stream(async_nats::jetstream::stream::Config {
        name: subject.name(),
        subjects: vec![
            subject.wildcard(),
        ],
        retention: RetentionPolicy::Interest,
        duplicate_window: std::time::Duration::from_secs(60 * 2),
        storage: StorageType::File,
        ..Default::default()
    }).await.context("create stream")?;

    loop {
        match watch_stream(&global, &subject).with_context(Context::global()).await.transpose() {
            Ok(None) => return Ok(()),
            Ok(Some(_)) => {
                tracing::warn!("stream closed, reconnecting");
            }
            Err(e) => {
                tracing::error!("failed to watch stream: {:#}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn watch_stream(global: &Arc<Global>, subject: &ChangeStreamSubject) -> anyhow::Result<()> {
    let mut stream = global.db.watch().await.context("watch database")?;
    while let Some(change) = stream.next().await.transpose()? {
        let mut errors = 0;
        loop {
            match process_change(global, subject, &change).await {
                Ok(_) => break,
                Err(e) => {
                    tracing::error!(error = %e, "failed to process change");
                    errors += 1;
                    if errors > 5 {
                        return Err(e);
                    }
                }
            }
        }
    }
    Ok(())
}

async fn process_change(global: &Arc<Global>, subject: &ChangeStreamSubject, change: &ChangeStreamEvent<mongodb::bson::Document>) -> anyhow::Result<()> {
    Ok(())
}