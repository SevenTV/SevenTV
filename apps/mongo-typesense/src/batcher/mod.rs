use scuffle_foundations::batcher::dataloader::DataLoader;
use scuffle_foundations::batcher::Batcher;
use serde::de::DeserializeOwned;
use serde::Serialize;
use shared::database::loader::LoaderById;
use shared::database::MongoCollection;
use shared::typesense::types::TypesenseCollection;
use typesense_insert::TypesenseInsert;

pub mod typesense_insert;
pub mod updater;

pub struct CollectionBatcher<
	M: MongoCollection + Clone + Serialize + DeserializeOwned + 'static,
	T: TypesenseCollection + serde::Serialize + 'static,
> {
	pub loader: DataLoader<LoaderById<M>>,
	pub inserter: Batcher<TypesenseInsert<T>>,
}

impl<
	M: MongoCollection + Clone + Serialize + DeserializeOwned + 'static,
	T: TypesenseCollection + serde::Serialize + 'static,
> CollectionBatcher<M, T>
{
	pub fn new(mongo: mongodb::Database, typesense: typesense_codegen::apis::configuration::Configuration) -> Self {
		Self {
			loader: LoaderById::new(mongo.clone()),
			inserter: TypesenseInsert::new(typesense),
		}
	}
}
