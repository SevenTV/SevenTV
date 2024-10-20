use std::sync::Arc;

use scuffle_foundations::batcher::{BatchMode, BatchOperation, Batcher, BatcherConfig, BatcherNormalMode};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::typesense::types::TypesenseCollection;
use typesense_rs::apis::documents_api::{ImportDocumentsError, ImportDocumentsParams, IndexDocumentError};
use typesense_rs::apis::Api;
use typesense_rs::models;

pub struct TypesenseInsert<T> {
	client: Arc<typesense_rs::apis::ApiClient>,
	config: BatcherConfig,
	_phantom: std::marker::PhantomData<T>,
}

impl<T: TypesenseCollection + serde::Serialize + 'static> TypesenseInsert<T> {
	pub fn new(client: Arc<typesense_rs::apis::ApiClient>) -> Batcher<Self> {
		Self::new_with_config(
			client,
			BatcherConfig {
				name: format!("TypesenseInsert<{}>", T::COLLECTION_NAME),
				concurrency: 500,
				max_batch_size: 10_000,
				sleep_duration: std::time::Duration::from_millis(100),
			},
		)
	}

	pub fn new_with_config(client: Arc<typesense_rs::apis::ApiClient>, config: BatcherConfig) -> Batcher<Self> {
		Batcher::new(Self {
			client,
			config,
			_phantom: std::marker::PhantomData,
		})
	}
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum TypesenseInsertError {
	#[error("failed to import documents: {0}")]
	Import(#[from] Arc<typesense_rs::apis::Error<ImportDocumentsError>>),
	#[error("insert document error")]
	Insert(IndexDocumentError),
	#[error("failed to serialize document: {0}")]
	Serialize(Arc<serde_json::Error>),
	#[error("failed to deserialize result: {0}")]
	Deserialize(Arc<serde_json::Error>),
}

impl<T: TypesenseCollection + serde::Serialize + 'static> BatchOperation for TypesenseInsert<T> {
	type Error = TypesenseInsertError;
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

		let body = documents
			.iter()
			.map(|d| serde_json::to_string(&d))
			.collect::<Result<Vec<_>, _>>()
			.map_err(Arc::new)
			.map_err(TypesenseInsertError::Serialize)?
			.join("\n");

		let r = self
			.client
			.documents_api()
			.import_documents(
				ImportDocumentsParams::builder()
					.collection_name(T::COLLECTION_NAME.to_owned())
					.action(models::IndexAction::Upsert)
					.body(body)
					.build(),
			)
			.await
			.map_err(Arc::new)?;

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

		Ok(r.lines()
			.map(|l| {
				let r = serde_json::from_str::<BatchInsertResultJson>(l);
				r.map_err(Arc::new).map_err(TypesenseInsertError::Deserialize)?.into_result()
			})
			.collect::<Vec<_>>())
	}
}
