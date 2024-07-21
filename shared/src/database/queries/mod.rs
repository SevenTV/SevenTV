use std::borrow::Borrow;

use mongodb::action::Multiple;

pub mod filter;
pub mod traits;
pub mod update;

pub struct TypedCollection<T: Send + Sync>(pub mongodb::Collection<T>);

pub trait CollectionExt<T: Send + Sync> {
	fn typed(self) -> TypedCollection<T>;
}

impl<T: Send + Sync> CollectionExt<T> for mongodb::Collection<T> {
	fn typed(self) -> TypedCollection<T> {
		TypedCollection(self)
	}
}

impl<T: Send + Sync> TypedCollection<T> {
	pub fn untyped(self) -> mongodb::Collection<T> {
		self.0
	}

	pub fn drop(&self) -> mongodb::action::DropCollection<'_> {
		self.0.drop()
	}

	pub fn create_index(&self, index: mongodb::IndexModel) -> mongodb::action::CreateIndex<'_> {
		self.0.create_index(index)
	}

	pub fn create_indexes(
		&self,
		indexes: impl IntoIterator<Item = mongodb::IndexModel>,
	) -> mongodb::action::CreateIndex<'_, Multiple> {
		self.0.create_indexes(indexes)
	}

	pub fn insert_one(&self, doc: impl Borrow<T>) -> mongodb::action::InsertOne<'_>
	where
		T: serde::Serialize,
	{
		self.0.insert_one(doc)
	}

	pub fn insert_many(&self, docs: impl IntoIterator<Item = impl Borrow<T>>) -> mongodb::action::InsertMany<'_>
	where
		T: serde::Serialize,
	{
		self.0.insert_many(docs)
	}

	pub fn find(&self, filter: impl Into<filter::Filter<T>>) -> mongodb::action::Find<'_, T>
	where
		T: serde::de::DeserializeOwned,
	{
		let filter = filter.into();
		self.0.find(filter.to_document())
	}

	pub fn find_one(&self, filter: impl Into<filter::Filter<T>>) -> mongodb::action::FindOne<'_, T>
	where
		T: serde::de::DeserializeOwned,
	{
		let filter = filter.into();
		self.0.find_one(filter.to_document())
	}

	pub fn find_one_and_update(
		&self,
		filter: impl Into<filter::Filter<T>>,
		update: impl Into<update::Update<T>>,
	) -> mongodb::action::FindOneAndUpdate<'_, T>
	where
		T: serde::de::DeserializeOwned,
	{
		let filter = filter.into();
		let update = update.into();
		self.0.find_one_and_update(filter.to_document(), update.to_document())
	}

	pub fn find_one_and_delete(&self, filter: impl Into<filter::Filter<T>>) -> mongodb::action::FindOneAndDelete<'_, T>
	where
		T: serde::de::DeserializeOwned,
	{
		let filter = filter.into();
		self.0.find_one_and_delete(filter.to_document())
	}

	pub fn update_many(
		&self,
		filter: impl Into<filter::Filter<T>>,
		update: impl Into<update::Update<T>>,
	) -> mongodb::action::Update<'_> {
		let filter = filter.into();
		let update = update.into();
		self.0.update_many(filter.to_document(), update.to_document())
	}

	pub fn update_one(
		&self,
		filter: impl Into<filter::Filter<T>>,
		update: impl Into<update::Update<T>>,
	) -> mongodb::action::Update<'_> {
		let filter = filter.into();
		let update = update.into();
		self.0.update_one(filter.to_document(), update.to_document())
	}

	pub fn delete_many(&self, filter: impl Into<filter::Filter<T>>) -> mongodb::action::Delete<'_> {
		let filter = filter.into();
		self.0.delete_many(filter.to_document())
	}

	pub fn delete_one(&self, filter: impl Into<filter::Filter<T>>) -> mongodb::action::Delete<'_> {
		let filter = filter.into();
		self.0.delete_one(filter.to_document())
	}
}
