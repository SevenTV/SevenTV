use std::sync::Arc;

use scuffle_foundations::batcher::{BatchMode, BatchOperation, Batcher, BatcherConfig, BatcherNormalMode};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::clickhouse::ClickhouseCollection;

pub struct ClickhouseInsert<T: ClickhouseCollection + serde::Serialize + 'static> {
	client: clickhouse::Client,
	config: BatcherConfig,
	_phantom: std::marker::PhantomData<T>,
}

impl<T: ClickhouseCollection + serde::Serialize + 'static> ClickhouseInsert<T> {
	pub fn new(client: clickhouse::Client) -> Batcher<Self> {
		Self::new_with_config(
			client,
			BatcherConfig {
				name: format!("ClickhouseInsert<{}>", T::COLLECTION_NAME),
				concurrency: 500,
				max_batch_size: 10_000,
				sleep_duration: std::time::Duration::from_millis(100),
			},
		)
	}

	pub fn new_with_config(client: clickhouse::Client, config: BatcherConfig) -> Batcher<Self> {
		Batcher::new(Self {
			client,
			config,
			_phantom: std::marker::PhantomData,
		})
	}
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum ClickhouseInsertError {
	#[error("failed to insert documents: {0}")]
	Import(#[from] Arc<clickhouse::error::Error>),
}

impl<T: ClickhouseCollection + serde::Serialize + 'static> BatchOperation for ClickhouseInsert<T> {
	type Error = ClickhouseInsertError;
	type Item = T;
	type Mode = BatcherNormalMode;
	type Response = bool;

	fn config(&self) -> BatcherConfig {
		let mut config = self.config.clone();
		config.name = format!("{}<{}>", config.name, T::COLLECTION_NAME);
		config
	}

	#[tracing::instrument(skip_all, fields(document_count = documents.len(), collection= T::COLLECTION_NAME))]
	async fn process(
		&self,
		documents: Vec<Self::Item>,
	) -> Result<<Self::Mode as BatchMode<Self>>::OperationOutput, Self::Error> {
		tracing::Span::current().make_root();

		let count = documents.len();

		let mut insert = self.client.insert::<T>(T::COLLECTION_NAME).map_err(Arc::new)?;

		for document in documents {
			insert.write(&document).await.map_err(Arc::new)?;
		}

		insert.end().await.map_err(Arc::new)?;

		Ok((0..count).map(|_| Ok(true)).collect())
	}
}
