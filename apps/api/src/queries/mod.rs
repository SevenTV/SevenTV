use std::marker::PhantomData;
use std::sync::Arc;

use futures::TryStreamExt;
use mongodb::error::{TRANSIENT_TRANSACTION_ERROR, UNKNOWN_TRANSACTION_COMMIT_RESULT};
use mongodb::results::{DeleteResult, UpdateResult};
use shared::database::audit_log::AuditLog;
use shared::database::MongoCollection;
use spin::Mutex;

use crate::global::Global;

/// TOOD(lennart): whatever this is supposed to be.
type EmittedEvent = ();

macro_rules! define_queries {
	($name:ident, $options:ty, $output:ty, $fn:ident) => {
		#[allow(dead_code, unused_variables)]
		pub trait $name {
			type Collection: MongoCollection + serde::de::DeserializeOwned;
			type Filter<'a>: serde::Serialize + 'a
			where
				Self: 'a;

			/// Filter to use for the fetch.
			fn filter(&self) -> Self::Filter<'_>;

			/// 	Options to use for the fetch.
			fn options(&self) -> Option<$options> {
				None
			}

			/// Executes the query using the given session.
			fn execute<T: Send>(
				&self,
				session: &mut TransactionSession<T>,
			) -> impl std::future::Future<Output = Result<$output, TransactionError<T>>> + Send
			where
				Self: Sized + Send + Sync,
			{
				session.$fn(self)
			}
		}
	};
	($name:ident, $options:ty, $output:ty, $fn:ident,update = $resp:ty) => {
		#[allow(dead_code, unused_variables)]
		pub trait $name {
			type Collection: MongoCollection + serde::de::DeserializeOwned;
			type Filter<'a>: serde::Serialize + 'a
			where
				Self: 'a;
			type Update<'a>: serde::Serialize + 'a
			where
				Self: 'a;

			/// Filter to use for the fetch.
			fn filter(&self) -> Self::Filter<'_>;

			/// Update's to apply
			fn update(&self) -> Self::Update<'_>;

			/// 	Options to use for the fetch.
			fn options(&self) -> Option<$options> {
				None
			}

			/// Events to emit
			fn emit_events(&self, resp: $resp) -> impl IntoIterator<Item = EmittedEvent> {
				std::iter::empty()
			}

			/// Audit logs to create
			fn audit_logs(&self, resp: $resp) -> impl IntoIterator<Item = AuditLog> {
				std::iter::empty()
			}

			/// Executes the query using the given session.
			fn execute<T: Send>(
				&self,
				session: &mut TransactionSession<T>,
			) -> impl std::future::Future<Output = Result<$output, TransactionError<T>>> + Send
			where
				Self: Sized + Send + Sync,
			{
				session.$fn(self)
			}
		}
	};
	($name:ident, $options:ty, $output:ty, $fn:ident,delete = $resp:ty) => {
		#[allow(dead_code, unused_variables)]
		pub trait $name {
			type Collection: MongoCollection + serde::de::DeserializeOwned;
			type Filter<'a>: serde::Serialize + 'a
			where
				Self: 'a;

			/// Filter to use for the fetch.
			fn filter(&self) -> Self::Filter<'_>;

			/// 	Options to use for the fetch.
			fn options(&self) -> Option<$options> {
				None
			}

			/// Events to emit
			fn emit_events(&self, resp: $resp) -> Vec<EmittedEvent> {
				Vec::new()
			}

			/// Audit logs to create
			fn audit_logs(&self, resp: $resp) -> Vec<AuditLog> {
				Vec::new()
			}

			/// Executes the query using the given session.
			fn execute<T: Send>(
				&self,
				session: &mut TransactionSession<T>,
			) -> impl std::future::Future<Output = Result<$output, TransactionError<T>>> + Send
			where
				Self: Sized + Send + Sync,
			{
				session.$fn(self)
			}
		}
	};
}

define_queries!(FindQuery, mongodb::options::FindOptions, Vec<Self::Collection>, find);

define_queries!(
	FindOneQuery,
	mongodb::options::FindOneOptions,
	Option<Self::Collection>,
	find_one
);

define_queries!(
	FindOneAndUpdateQuery,
	mongodb::options::FindOptions,
	Option<Self::Collection>,
	find_one_and_update,
	update = Option<&Self::Collection>
);

define_queries!(
	UpdateQuery,
	mongodb::options::UpdateOptions,
	UpdateResult,
	update,
	update = &UpdateResult
);

define_queries!(
	UpdateOneQuery,
	mongodb::options::UpdateOptions,
	UpdateResult,
	update_one,
	update = &UpdateResult
);

define_queries!(
	DeleteQuery,
	mongodb::options::DeleteOptions,
	DeleteResult,
	delete,
	delete = &DeleteResult
);

define_queries!(
	FindOneAndDeleteQuery,
	mongodb::options::FindOneAndDeleteOptions,
	Option<Self::Collection>,
	find_one_and_delete,
	delete = Option<&Self::Collection>
);

#[derive(Debug, serde::Serialize)]
pub struct MongoSet<T> {
	#[serde(rename = "$set")]
	pub set: T,
}

#[derive(Debug, serde::Serialize)]
pub struct MongoPull<T> {
	#[serde(rename = "$pull")]
	pub pull: T,
}

#[derive(Debug, serde::Serialize)]
pub struct MongoPush<T> {
	#[serde(rename = "$push")]
	pub push: T,
}

#[derive(Debug, serde::Serialize)]
pub struct MongoIn<T> {
	#[serde(rename = "$in")]
	pub in_: T,
}

pub struct TransactionSession<T>(Arc<Mutex<SessionInner>>, PhantomData<T>); // 

impl<T> TransactionSession<T> {
	fn new(inner: Arc<Mutex<SessionInner>>) -> Self {
		Self(inner, PhantomData)
	}

	fn reset(&mut self) -> Result<(), TransactionError<T>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;
		this.events.clear();
		Ok(())
	}

	fn clone(&self) -> Self {
		Self(self.0.clone(), PhantomData)
	}
}

impl<T> TransactionSession<T> {
	pub async fn find<U: FindQuery>(&mut self, find: &U) -> Result<Vec<U::Collection>, TransactionError<T>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let filter = bson::to_document(&find.filter()).map_err(TransactionError::Filter)?;

		let mut find = <U::Collection as MongoCollection>::collection(this.global.db())
			.find(filter)
			.with_options(find.options())
			.session(&mut this.session)
			.await?;

		Ok(find.stream(&mut this.session).try_collect().await?)
	}

	pub async fn find_one<U: FindOneQuery>(&mut self, find_one: &U) -> Result<Option<U::Collection>, TransactionError<T>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let filter = bson::to_document(&find_one.filter()).map_err(TransactionError::Filter)?;

		let result = <U::Collection as MongoCollection>::collection(this.global.db())
			.find_one(filter)
			.with_options(find_one.options())
			.session(&mut this.session)
			.await
			.map_err(TransactionError::Mongo)?;

		Ok(result)
	}

	pub async fn find_one_and_update<U: FindOneAndUpdateQuery>(
		&mut self,
		find_one_and_update: &U,
	) -> Result<Option<U::Collection>, TransactionError<T>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let filter = bson::to_document(&find_one_and_update.filter()).map_err(TransactionError::Filter)?;
		let update = bson::to_document(&find_one_and_update.update()).map_err(TransactionError::Update)?;

		let result = <U::Collection as MongoCollection>::collection(this.global.db())
			.find_one_and_update(filter, update)
			.session(&mut this.session)
			.await?;

		AuditLog::collection(this.global.db())
			.insert_many(find_one_and_update.audit_logs(result.as_ref()))
			.session(&mut this.session)
			.await?;

		this.events.extend(find_one_and_update.emit_events(result.as_ref()));

		Ok(result)
	}

	pub async fn find_one_and_delete<U: FindOneAndDeleteQuery>(
		&mut self,
		find_one_and_delete: &U,
	) -> Result<Option<U::Collection>, TransactionError<T>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let filter = bson::to_document(&find_one_and_delete.filter()).map_err(TransactionError::Filter)?;

		let result = <U::Collection as MongoCollection>::collection(this.global.db())
			.find_one_and_delete(filter)
			.session(&mut this.session)
			.await
			.map_err(TransactionError::Mongo)?;

		AuditLog::collection(this.global.db())
			.insert_many(find_one_and_delete.audit_logs(result.as_ref()))
			.session(&mut this.session)
			.await?;

		this.events.extend(find_one_and_delete.emit_events(result.as_ref()));

		Ok(result)
	}

	pub async fn update<U: UpdateQuery>(&mut self, update: &U) -> Result<UpdateResult, TransactionError<T>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let filter = bson::to_document(&update.filter()).map_err(TransactionError::Filter)?;
		let update_ = bson::to_document(&update.update()).map_err(TransactionError::Update)?;

		let result = <U::Collection as MongoCollection>::collection(this.global.db())
			.update_many(filter, update_)
			.session(&mut this.session)
			.await?;

		AuditLog::collection(this.global.db())
			.insert_many(update.audit_logs(&result))
			.session(&mut this.session)
			.await?;

		this.events.extend(update.emit_events(&result));

		Ok(result)
	}

	pub async fn update_one<U: UpdateOneQuery>(&mut self, update_one: &U) -> Result<UpdateResult, TransactionError<T>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let filter = bson::to_document(&update_one.filter()).map_err(TransactionError::Filter)?;
		let update_ = bson::to_document(&update_one.update()).map_err(TransactionError::Update)?;

		let result = <U::Collection as MongoCollection>::collection(this.global.db())
			.update_one(filter, update_)
			.session(&mut this.session)
			.await?;

		AuditLog::collection(this.global.db())
			.insert_many(update_one.audit_logs(&result))
			.session(&mut this.session)
			.await?;

		this.events.extend(update_one.emit_events(&result));

		Ok(result)
	}

	pub async fn delete<U: DeleteQuery>(&mut self, delete: &U) -> Result<DeleteResult, TransactionError<T>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let filter = bson::to_document(&delete.filter()).map_err(TransactionError::Filter)?;

		let result = <U::Collection as MongoCollection>::collection(this.global.db())
			.delete_many(filter)
			.session(&mut this.session)
			.await?;

		AuditLog::collection(this.global.db())
			.insert_many(delete.audit_logs(&result))
			.session(&mut this.session)
			.await?;

		this.events.extend(delete.emit_events(&result));

		Ok(result)
	}
}

struct SessionInner {
	global: Arc<Global>,
	session: mongodb::ClientSession,
	events: Vec<EmittedEvent>,
}

#[derive(thiserror::Error, Debug)]
pub enum TransactionError<T> {
	#[error("mongo error: {0}")]
	Mongo(#[from] mongodb::error::Error),
	#[error("session locked after returning")]
	SessionLocked,
	#[error("filter error: {0}")]
	Filter(bson::ser::Error),
	#[error("modifier error: {0}")]
	Update(bson::ser::Error),
	#[error("custom error")]
	Custom(T),
}

impl<T, E> TransactionError<Result<T, E>> {
	pub fn custom(err: E) -> Self {
		Self::Custom(Err(err))
	}
}

#[derive(Debug, Clone)]
pub struct TransactionOutput<T> {
	pub output: T,
	pub aborted: bool,
}

impl<T> std::ops::Deref for TransactionOutput<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.output
	}
}

impl<T> std::ops::DerefMut for TransactionOutput<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.output
	}
}

pub async fn with_transaction<T, F, Fut>(global: Arc<Global>, f: F) -> Result<TransactionOutput<T>, TransactionError<T>>
where
	F: FnOnce(TransactionSession<T>) -> Fut + Clone,
	Fut: std::future::Future<Output = Result<T, TransactionError<T>>>,
{
	let mut session = global.mongo().start_session().await?;
	session.start_transaction().await?;

	let mut session = TransactionSession::new(Arc::new(Mutex::new(SessionInner {
		global,
		session,
		events: Vec::new(),
	})));

	'retry_operation: loop {
		session.reset()?;
		let result = (f.clone())(session.clone()).await;
		let mut session_inner = session.0.try_lock().ok_or(TransactionError::SessionLocked)?;
		match result {
			Ok(output) => 'retry_commit: loop {
				match session_inner.session.commit_transaction().await {
					Ok(()) => {
						// todo emit events
						return Ok(TransactionOutput { output, aborted: false });
					}
					Err(err) => {
						if err.contains_label(UNKNOWN_TRANSACTION_COMMIT_RESULT) {
							continue 'retry_commit;
						} else if err.contains_label(TRANSIENT_TRANSACTION_ERROR) {
							continue 'retry_operation;
						}

						return Err(TransactionError::Mongo(err));
					}
				}
			},
			Err(err) => {
				if let TransactionError::Mongo(err) = &err {
					if err.contains_label(TRANSIENT_TRANSACTION_ERROR) {
						continue 'retry_operation;
					}
				}

				session_inner.session.abort_transaction().await?;

				if let TransactionError::Custom(output) = err {
					return Ok(TransactionOutput { output, aborted: true });
				}

				return Err(err);
			}
		}
	}
}
