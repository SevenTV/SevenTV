use std::borrow::Borrow;
use std::marker::PhantomData;
use std::sync::Arc;

use futures::TryStreamExt;
use mongodb::error::{TRANSIENT_TRANSACTION_ERROR, UNKNOWN_TRANSACTION_COMMIT_RESULT};
use mongodb::results::{DeleteResult, InsertManyResult, InsertOneResult, UpdateResult};
use shared::database::queries::{filter, update};
use shared::database::MongoCollection;
use spin::Mutex;

use crate::global::Global;

/// TOOD(lennart): whatever this is supposed to be.
type EmittedEvent = ();

pub struct TransactionSession<'a, E>(Arc<Mutex<SessionInner<'a>>>, PhantomData<E>);

impl<'a, E> TransactionSession<'a, E> {
	fn new(inner: Arc<Mutex<SessionInner<'a>>>) -> Self {
		Self(inner, PhantomData)
	}

	fn reset(&mut self) -> Result<(), TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;
		this.events.clear();
		Ok(())
	}

	fn clone(&self) -> Self {
		Self(self.0.clone(), PhantomData)
	}
}

impl<E> TransactionSession<'_, E> {
	pub async fn find<U: MongoCollection + serde::de::DeserializeOwned>(
		&mut self,
		filter: impl Into<filter::Filter<U>>,
		options: impl Into<Option<mongodb::options::FindOptions>>,
	) -> Result<Vec<U>, TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let mut find = U::collection(&this.global.db)
			.find(filter)
			.with_options(options)
			.session(&mut this.session)
			.await?;

		Ok(find.stream(&mut this.session).try_collect().await?)
	}

	pub async fn find_one<U: MongoCollection + serde::de::DeserializeOwned>(
		&mut self,
		filter: impl Into<filter::Filter<U>>,
		options: impl Into<Option<mongodb::options::FindOneOptions>>,
	) -> Result<Option<U>, TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let result = U::collection(&this.global.db)
			.find_one(filter)
			.with_options(options)
			.session(&mut this.session)
			.await
			.map_err(TransactionError::Mongo)?;

		Ok(result)
	}

	pub async fn find_one_and_update<U: MongoCollection + serde::de::DeserializeOwned>(
		&mut self,
		filter: impl Into<filter::Filter<U>>,
		update: impl Into<update::Update<U>>,
		options: impl Into<Option<mongodb::options::FindOneAndUpdateOptions>>,
	) -> Result<Option<U>, TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let result = U::collection(&this.global.db)
			.find_one_and_update(filter, update)
			.with_options(options)
			.session(&mut this.session)
			.await?;

		Ok(result)
	}

	pub async fn find_one_and_delete<U: MongoCollection + serde::de::DeserializeOwned>(
		&mut self,
		filter: impl Into<filter::Filter<U>>,
		options: impl Into<Option<mongodb::options::FindOneAndDeleteOptions>>,
	) -> Result<Option<U>, TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let result = U::collection(&this.global.db)
			.find_one_and_delete(filter)
			.with_options(options)
			.session(&mut this.session)
			.await
			.map_err(TransactionError::Mongo)?;

		Ok(result)
	}

	pub async fn update<U: MongoCollection>(
		&mut self,
		filter: impl Into<filter::Filter<U>>,
		update: impl Into<update::Update<U>>,
		options: impl Into<Option<mongodb::options::UpdateOptions>>,
	) -> Result<UpdateResult, TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let result = U::collection(&this.global.db)
			.update_many(filter, update)
			.with_options(options)
			.session(&mut this.session)
			.await?;

		Ok(result)
	}

	pub async fn update_one<U: MongoCollection>(
		&mut self,
		filter: impl Into<filter::Filter<U>>,
		update: impl Into<update::Update<U>>,
		options: impl Into<Option<mongodb::options::UpdateOptions>>,
	) -> Result<UpdateResult, TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let result = U::collection(&this.global.db)
			.update_one(filter, update)
			.with_options(options)
			.session(&mut this.session)
			.await?;

		Ok(result)
	}

	pub async fn delete<U: MongoCollection>(
		&mut self,
		filter: impl Into<filter::Filter<U>>,
		options: impl Into<Option<mongodb::options::DeleteOptions>>,
	) -> Result<DeleteResult, TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let result = U::collection(&this.global.db)
			.delete_many(filter)
			.with_options(options)
			.session(&mut this.session)
			.await?;

		Ok(result)
	}

	pub async fn delete_one<U: MongoCollection>(
		&mut self,
		filter: impl Into<filter::Filter<U>>,
		options: impl Into<Option<mongodb::options::DeleteOptions>>,
	) -> Result<DeleteResult, TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let result = U::collection(&this.global.db)
			.delete_one(filter)
			.with_options(options)
			.session(&mut this.session)
			.await?;

		Ok(result)
	}

	pub async fn count<U: MongoCollection>(
		&mut self,
		filter: impl Into<filter::Filter<U>>,
		options: impl Into<Option<mongodb::options::CountOptions>>,
	) -> Result<u64, TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let result = U::collection(&this.global.db)
			.count_documents(filter)
			.with_options(options)
			.session(&mut this.session)
			.await
			.map_err(TransactionError::Mongo)?;

		Ok(result)
	}

	pub async fn insert_one<U: MongoCollection + serde::Serialize>(
		&mut self,
		insert: impl Borrow<U>,
		options: impl Into<Option<mongodb::options::InsertOneOptions>>,
	) -> Result<InsertOneResult, TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let result = U::collection(&this.global.db)
			.insert_one(insert)
			.with_options(options)
			.session(&mut this.session)
			.await?;

		Ok(result)
	}

	pub async fn insert_many<U: MongoCollection + serde::Serialize>(
		&mut self,
		items: impl IntoIterator<Item = impl Borrow<U>>,
		options: impl Into<Option<mongodb::options::InsertManyOptions>>,
	) -> Result<InsertManyResult, TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;

		let result = U::collection(&this.global.db)
			.insert_many(items)
			.with_options(options)
			.session(&mut this.session)
			.await?;

		Ok(result)
	}

	pub fn register_event(&mut self, event: EmittedEvent) -> Result<(), TransactionError<E>> {
		let mut this = self.0.try_lock().ok_or(TransactionError::SessionLocked)?;
		this.events.push(event);
		Ok(())
	}
}

struct SessionInner<'a> {
	global: &'a Arc<Global>,
	session: mongodb::ClientSession,
	events: Vec<EmittedEvent>,
}

#[derive(thiserror::Error, Debug)]
pub enum TransactionError<E> {
	#[error("mongo error: {0}")]
	Mongo(#[from] mongodb::error::Error),
	#[error("session locked after returning")]
	SessionLocked,
	#[error("filter error: {0}")]
	Filter(bson::ser::Error),
	#[error("modifier error: {0}")]
	Update(bson::ser::Error),
	#[error("custom error")]
	Custom(E),
	#[error("too many failures")]
	TooManyFailures,
}

pub type TransactionResult<T, E> = Result<T, TransactionError<E>>;

impl<E> TransactionError<E> {
	pub const fn custom(err: E) -> Self {
		Self::Custom(err)
	}
}

// #[derive(Debug, Clone)]
// pub struct TransactionOutput<T> {
// 	pub output: T,
// 	pub aborted: bool,
// }

// impl<T> TransactionOutput<T> {
// 	pub fn into_inner(self) -> T {
// 		self.output
// 	}
// }

// impl<T> std::ops::Deref for TransactionOutput<T> {
// 	type Target = T;

// 	fn deref(&self) -> &Self::Target {
// 		&self.output
// 	}
// }

// impl<T> std::ops::DerefMut for TransactionOutput<T> {
// 	fn deref_mut(&mut self) -> &mut Self::Target {
// 		&mut self.output
// 	}
// }

pub async fn with_transaction<'a, T, E, F, Fut>(global: &'a Arc<Global>, f: F) -> TransactionResult<T, E>
where
	F: FnOnce(TransactionSession<'a, E>) -> Fut + Clone + 'a,
	Fut: std::future::Future<Output = TransactionResult<T, E>> + 'a,
{
	let mut session = global.mongo.start_session().await?;
	session.start_transaction().await?;

	let mut session = TransactionSession::new(Arc::new(Mutex::new(SessionInner {
		global,
		session,
		events: Vec::new(),
	})));

	let mut retry_count = 0;

	'retry_operation: loop {
		if retry_count > 3 {
			return Err(TransactionError::TooManyFailures);
		}

		retry_count += 1;
		session.reset()?;
		let result = (f.clone())(session.clone()).await;
		let mut session_inner = session.0.try_lock().ok_or(TransactionError::SessionLocked)?;
		match result {
			Ok(output) => 'retry_commit: loop {
				match session_inner.session.commit_transaction().await {
					Ok(()) => {
						// todo emit events
						session_inner.events.clear();
						return Ok(output);
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

				// if let TransactionError::Custom(output) = err {
				// 	return Ok(TransactionOutput { output, aborted: true });
				// }

				return Err(err);
			}
		}
	}
}
