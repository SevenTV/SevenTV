use scuffle_foundations::batcher::dataloader::DataLoader;
use scuffle_foundations::batcher::Batcher;
use serde::de::DeserializeOwned;
use shared::database::loader::LoaderById;
use shared::database::SearchableMongoCollection;
use shared::typesense::types::TypesenseCollection;
use typesense_insert::TypesenseInsert;

pub mod clickhouse;
pub mod typesense_insert;
pub mod updater;

pub struct CollectionBatcher<M: SearchableMongoCollection>
where
	M: DeserializeOwned + Clone + 'static,
	M::Typesense: TypesenseCollection + serde::Serialize + 'static,
{
	pub loader: DataLoader<LoaderById<M>>,
	pub inserter: Batcher<TypesenseInsert<M::Typesense>>,
}

impl<M: SearchableMongoCollection> CollectionBatcher<M>
where
	M: DeserializeOwned + Clone + 'static,
	M::Typesense: TypesenseCollection + serde::Serialize + 'static,
{
	pub fn new(mongo: mongodb::Database, typesense: typesense_codegen::apis::configuration::Configuration) -> Self {
		Self {
			loader: LoaderById::new(mongo.clone()),
			inserter: TypesenseInsert::new(typesense),
		}
	}
}
