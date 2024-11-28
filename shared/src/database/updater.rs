use std::collections::HashMap;

use scuffle_batching::batch::BatchResponse;
use scuffle_batching::{BatchExecutor, Batcher};

use crate::database::queries::{filter, update};
use crate::database::MongoCollection;

pub struct MongoUpdater(Batcher<Inner>);

struct Inner {
	db: mongodb::Database,
}

impl MongoUpdater {
	pub fn new(db: mongodb::Database, batch_size: usize, delay: std::time::Duration) -> Self {
		Self(Batcher::new(Inner { db }, batch_size, delay))
	}

	pub async fn update<M: MongoCollection>(
		&self,
		filter: impl Into<filter::Filter<M>>,
		update: impl Into<update::Update<M>>,
		many: bool,
	) -> Result<bool, MongoOpError> {
		self.bulk(Some(MongoReq::update::<M>(filter, update, many)))
			.await
			.pop()
			.ok_or(MongoOpError::NoResponse)?
	}

	pub async fn bulk(
		&self,
		requests: impl IntoIterator<Item = MongoReq> + Send,
	) -> Vec<Result<bool, MongoOpError>> {
		self.0
			.execute_many(requests)
			.await
			.into_iter()
			.map(|r| r.unwrap_or(Err(MongoOpError::NoResponse)))
			.collect()
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

impl BatchExecutor for Inner {
	type Request = MongoReq;
	type Response = Result<bool, MongoOpError>;

	async fn execute(
		&self,
		documents: Vec<(Self::Request, BatchResponse<Self::Response>)>,
	) {
		let mut collections = HashMap::new();

		let (docs, callbacks) = documents
			.into_iter()
			.map(|(req, resp)| {
				if !collections.contains_key(&req.collection) {
					collections.insert(req.collection, collections.len());
				}

				let idx = collections[&req.collection] as u32;

				let op = match req.op {
					MongoOp::Update { filter, update, many } => bson::doc! {
						"update": idx,
						"filter": filter,
						"updateMods": update,
						"multi": many,
					},
				};

				(op, resp)
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

		let r = match self
			.db
			.client()
			.database("admin")
			.run_command(bson::doc! {
				"bulkWrite": 1,
				"ordered": false,
				"ops": docs,
				"nsInfo": ns_info,
			})
			.await {
				Ok(r) => r,
				Err(e) => {
					tracing::error!("failed to bulk write: {e}");
					callbacks.into_iter().for_each(|c| c.send_err(MongoOpError::Import(e.clone())));
					return;
				}
			};

		let resp: BulkWriteResp = match bson::from_document(r) {
			Ok(r) => r,
			Err(e) => {
				tracing::error!("failed to deserialize bulk write response: {e}");
				callbacks.into_iter().for_each(|c| c.send_err(MongoOpError::Deserialize(e.clone())));
				return;
			}
		};

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

		for (idx, callback) in callbacks.into_iter().enumerate() {
			let update = match resp_batch.get(&(idx as u32)) {
				Some(r) => r,
				None => {
					callback.send_err(MongoOpError::NoResponse);
					continue;
				}
			};

			if update.ok >= 1.0 {
				callback.send_ok(update.n_modified.unwrap_or(update.n) > 0);
			} else {
				callback.send_err(MongoOpError::Response(
					update.code.unwrap_or(1),
					update.errmsg.clone().unwrap_or_default(),
				));
			}
		}
	}
}
