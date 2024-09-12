use std::collections::HashMap;

use scuffle_foundations::batcher::{BatchMode, BatchOperation, Batcher, BatcherConfig, BatcherError, BatcherNormalMode};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;

use crate::database::queries::{filter, update};
use crate::database::MongoCollection;

pub struct MongoUpdater(Batcher<Inner>);

struct Inner {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl MongoUpdater {
	pub fn new(db: mongodb::Database, config: BatcherConfig) -> Self {
		Self(Batcher::new(Inner { db, config }))
	}

	pub async fn update<M: MongoCollection>(
		&self,
		filter: impl Into<filter::Filter<M>>,
		update: impl Into<update::Update<M>>,
		many: bool,
	) -> Result<bool, BatcherError<MongoOpError>> {
		self.bulk(Some(MongoReq::update::<M>(filter, update, many)))
			.await
			.pop()
			.ok_or(BatcherError::MissingResult)?
	}

	pub async fn bulk(
		&self,
		requests: impl IntoIterator<Item = MongoReq> + Send,
	) -> Vec<Result<bool, BatcherError<MongoOpError>>> {
		self.0.execute_many(requests).await
	}
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum MongoOpError {
	#[error("failed to import documents: {0}")]
	Import(#[from] mongodb::error::Error),
	#[error("failed to serialize document: {0}")]
	Serialize(#[from] bson::ser::Error),
	#[error("failed to deserialize result: {0}")]
	Deserialize(#[from] bson::de::Error),
	#[error("mongo response error: [{0}] {1}")]
	Response(u32, String),
	#[error("no response")]
	NoResponse,
}

#[derive(Debug, Clone)]
pub struct MongoReq {
	collection: &'static str,
	op: MongoOp,
}

impl MongoReq {
	pub fn update<M: MongoCollection>(
		filter: impl Into<filter::Filter<M>>,
		update: impl Into<update::Update<M>>,
		many: bool,
	) -> Self {
		Self {
			collection: M::COLLECTION_NAME,
			op: MongoOp::Update {
				filter: filter.into().to_document(),
				update: update.into().to_document(),
				many,
			},
		}
	}
}

#[derive(Debug, Clone)]
enum MongoOp {
	Update {
		filter: bson::Document,
		update: bson::Document,
		many: bool,
	},
}

impl BatchOperation for Inner {
	type Error = MongoOpError;
	type Item = MongoReq;
	type Mode = BatcherNormalMode;
	type Response = bool;

	fn config(&self) -> BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(document_count = documents.len()))]
	async fn process(
		&self,
		documents: Vec<Self::Item>,
	) -> Result<<Self::Mode as BatchMode<Self>>::OperationOutput, Self::Error> {
		tracing::Span::current().make_root();

		let mut collections = HashMap::new();

		let ops = documents
			.into_iter()
			.map(|req| {
				if !collections.contains_key(&req.collection) {
					collections.insert(req.collection, collections.len());
				}

				let idx = collections[&req.collection] as u32;

				Ok::<_, MongoOpError>(match req.op {
					MongoOp::Update { filter, update, many } => bson::doc! {
						"update": idx,
						"filter": filter,
						"updateMods": update,
						"multi": many,
					},
				})
			})
			.collect::<Vec<_>>();

		let (true_idx_map, docs) = ops
			.iter()
			.enumerate()
			.filter_map(|(idx, r)| match r {
				Ok(r) => Some((idx, r)),
				Err(_) => None,
			})
			.unzip::<_, _, Vec<_>, Vec<_>>();

		let mut collections = collections
			.into_iter()
			.map(|(k, v)| {
				(
					bson::doc! {
						"ns": format!("{}.{}", self.db.name(), k),
					},
					v,
				)
			})
			.collect::<Vec<_>>();

		collections.sort_by_key(|(_, v)| *v);

		let ns_info = collections.into_iter().map(|(v, _)| v).collect::<Vec<_>>();

		let r = self
			.db
			.client()
			.database("admin")
			.run_command(bson::doc! {
				"bulkWrite": 1,
				"ordered": false,
				"ops": docs,
				"nsInfo": ns_info,
			})
			.await
			.map_err(MongoOpError::Import)?;

		let resp: BulkWriteResp = bson::from_document(r)?;

		#[derive(Debug, serde::Deserialize)]
		struct BulkWriteResp {
			cursor: BulkWriteCursor,
		}

		#[derive(Debug, serde::Deserialize)]
		struct BulkWriteCursor {
			#[serde(rename = "firstBatch")]
			fist_batch: Vec<BulkWriteResult>,
		}

		#[derive(Debug, serde::Deserialize)]
		struct BulkWriteResult {
			ok: f64,
			idx: u32,
			n: u32,
			#[serde(default, rename = "nModified")]
			n_modified: Option<u32>,
			#[serde(default)]
			code: Option<u32>,
			#[serde(default)]
			errmsg: Option<String>,
		}

		let resp_batch = resp
			.cursor
			.fist_batch
			.into_iter()
			.map(|r| (r.idx, r))
			.collect::<HashMap<_, _>>();

		let mut results = Vec::with_capacity(ops.len());

		for (idx, r) in ops.into_iter().enumerate() {
			if let Err(e) = r {
				results.push(Err(e));
				continue;
			}

			let updated_idx = true_idx_map.binary_search(&idx).unwrap() as u32;
			if let Some(r) = resp_batch.get(&updated_idx) {
				if r.ok >= 1.0 {
					results.push(Ok(r.n_modified.unwrap_or(r.n) > 0));
				} else {
					results.push(Err(MongoOpError::Response(
						r.code.unwrap_or(1),
						r.errmsg.clone().unwrap_or_default(),
					)));
				}
			} else {
				results.push(Err(MongoOpError::NoResponse));
			}
		}

		Ok(results)
	}
}
