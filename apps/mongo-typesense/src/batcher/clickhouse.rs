use std::sync::Arc;

use scuffle_batching::batch::BatchResponse;
use scuffle_batching::{BatchExecutor, Batcher};
use shared::clickhouse::ClickhouseCollection;

pub struct ClickhouseInsert<T: ClickhouseCollection + serde::Serialize + 'static> {
	client: clickhouse::Client,
	_phantom: std::marker::PhantomData<T>,
}

impl<T: ClickhouseCollection + serde::Serialize + 'static> ClickhouseInsert<T> {
	pub fn new(client: clickhouse::Client) -> Batcher<Self> {
		Self::new_with_config(client, 10_000, 500, std::time::Duration::from_millis(100))
	}

	pub fn new_with_config(
		client: clickhouse::Client,
		batch_size: usize,
		concurrency: usize,
		delay: std::time::Duration,
	) -> Batcher<Self> {
		Batcher::new(
			Self {
				client,
				_phantom: std::marker::PhantomData,
			},
			batch_size,
			concurrency,
			delay,
		)
	}
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum ClickhouseInsertError {
	#[error("failed to insert documents: {0}")]
	Import(#[from] Arc<clickhouse::error::Error>),
}

impl<T: ClickhouseCollection + serde::Serialize + 'static> BatchExecutor for ClickhouseInsert<T> {
	type Request = T;
	type Response = Result<(), ClickhouseInsertError>;

	async fn execute(&self, documents: Vec<(Self::Request, BatchResponse<Self::Response>)>) {
		let mut insert = match self.client.insert::<T>(T::COLLECTION_NAME) {
			Ok(insert) => insert,
			Err(e) => {
				let err = ClickhouseInsertError::Import(Arc::new(e));
				documents.into_iter().for_each(|(_, send)| send.send_err(err.clone()));
				return;
			}
		};

		let mut senders = Vec::new();

		for (document, send) in documents {
			match insert.write(&document).await {
				Ok(_) => senders.push(send),
				Err(e) => send.send_err(ClickhouseInsertError::Import(Arc::new(e))),
			}
		}

		let r = insert.end().await.map_err(|e| ClickhouseInsertError::Import(Arc::new(e)));
		senders.into_iter().for_each(|s| s.send(r.clone()));
	}
}
