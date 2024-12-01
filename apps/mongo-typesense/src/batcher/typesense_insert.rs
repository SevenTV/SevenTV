use std::sync::Arc;

use scuffle_batching::batch::BatchResponse;
use scuffle_batching::{BatchExecutor, Batcher};
use shared::typesense::types::TypesenseCollection;
use typesense_rs::apis::documents_api::{ImportDocumentsError, ImportDocumentsParams, IndexDocumentError};
use typesense_rs::apis::Api;
use typesense_rs::models;

pub struct TypesenseInsert<T> {
	client: Arc<typesense_rs::apis::ApiClient>,
	_phantom: std::marker::PhantomData<T>,
}

impl<T: TypesenseCollection + serde::Serialize + 'static> TypesenseInsert<T> {
	pub fn new(client: Arc<typesense_rs::apis::ApiClient>) -> Batcher<Self> {
		Self::new_with_config(client, 500, 10_000, std::time::Duration::from_millis(100))
	}

	pub fn new_with_config(
		client: Arc<typesense_rs::apis::ApiClient>,
		concurrency: usize,
		max_batch_size: usize,
		sleep_duration: std::time::Duration,
	) -> Batcher<Self> {
		Batcher::new(
			Self {
				client,
				_phantom: std::marker::PhantomData,
			},
			max_batch_size,
			concurrency,
			sleep_duration,
		)
	}
}

#[derive(thiserror::Error, Debug)]
pub enum TypesenseInsertError {
	#[error("failed to import documents: {0}")]
	Import(#[from] Arc<typesense_rs::apis::Error<ImportDocumentsError>>),
	#[error("insert document error")]
	Insert(IndexDocumentError),
	#[error("failed to serialize document: {0}")]
	Serialize(serde_json::Error),
	#[error("failed to deserialize result: {0}")]
	Deserialize(serde_json::Error),
}

impl<T: TypesenseCollection + serde::Serialize + 'static> BatchExecutor for TypesenseInsert<T> {
	type Request = T;
	type Response = Result<bool, TypesenseInsertError>;

	async fn execute(&self, documents: Vec<(Self::Request, BatchResponse<Self::Response>)>) {
		let (body, responses) = documents
			.into_iter()
			.filter_map(|(d, send)| {
				let result = serde_json::to_string(&d);
				match result {
					Ok(result) => Some((result, send)),
					Err(e) => {
						send.send_err(TypesenseInsertError::Serialize(e));
						None
					}
				}
			})
			.unzip::<_, _, Vec<_>, Vec<_>>();

		let r = match self
			.client
			.documents_api()
			.import_documents(
				ImportDocumentsParams::builder()
					.collection_name(T::COLLECTION_NAME.to_owned())
					.action(models::IndexAction::Upsert)
					.body(body.join("\n"))
					.build(),
			)
			.await
		{
			Ok(r) => r,
			Err(e) => {
				let err = Arc::new(e);
				responses.into_iter().for_each(|r| r.send_err(err.clone().into()));
				return;
			}
		};

		#[derive(serde::Deserialize)]
		#[serde(untagged)]
		enum BatchInsertResultJson {
			Success { success: bool },
			Error(IndexDocumentError),
		}

		impl BatchInsertResultJson {
			fn into_result(self) -> Result<bool, TypesenseInsertError> {
				match self {
					BatchInsertResultJson::Success { success } => Ok(success),
					BatchInsertResultJson::Error(e) => Err(TypesenseInsertError::Insert(e)),
				}
			}
		}

		for (send, response) in responses.into_iter().zip(r.lines()) {
			match serde_json::from_str::<BatchInsertResultJson>(response) {
				Ok(result) => send.send(result.into_result()),
				Err(e) => send.send_err(TypesenseInsertError::Deserialize(e)),
			}
		}
	}
}
