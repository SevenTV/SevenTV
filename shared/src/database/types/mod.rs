pub mod automod;
pub mod badge;
pub mod cron_job;
pub mod duration;
pub mod emote;
pub mod emote_moderation_request;
pub mod emote_set;
pub mod entitlement;
pub mod global;
pub mod image_set;
pub mod page;
pub mod paint;
pub mod product;
pub mod role;
pub mod stored_event;
pub mod ticket;
pub mod user;
pub mod webhook_event;

pub use macros::MongoCollection;
pub use mongodb;

use super::queries::{CollectionExt, TypedCollection};
use crate::typesense::types::TypesenseCollection;

pub trait MongoCollection: Send + Sync {
	type Id: std::fmt::Debug
		+ Clone
		+ Eq
		+ std::hash::Hash
		+ serde::Serialize
		+ serde::de::DeserializeOwned
		+ Send
		+ Sync
		+ 'static;

	const COLLECTION_NAME: &'static str;

	fn id(&self) -> Self::Id;

	fn collection(db: &mongodb::Database) -> TypedCollection<Self>
	where
		Self: Sized,
	{
		db.collection(Self::COLLECTION_NAME).typed()
	}

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![]
	}
}

pub trait SearchableMongoCollection: MongoCollection {
	type Typesense: TypesenseCollection;
}

struct MongoGenericCollection {
	name: &'static str,
	indexes: Vec<mongodb::IndexModel>,
}

impl MongoGenericCollection {
	fn new<C: MongoCollection>() -> Self {
		Self {
			name: C::COLLECTION_NAME,
			indexes: C::indexes(),
		}
	}

	async fn init(self, db: &mongodb::Database) -> anyhow::Result<()> {
		let collection = db.collection::<()>(self.name);

		for index in self.indexes {
			collection.create_index(index).await?;
		}

		Ok(())
	}
}

fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	std::iter::empty()
		.chain(stored_event::mongo_collections())
		.chain(automod::mongo_collections())
		.chain(badge::mongo_collections())
		.chain(emote::mongo_collections())
		.chain(emote_set::mongo_collections())
		.chain(entitlement::mongo_collections())
		.chain(global::mongo_collections())
		.chain(page::mongo_collections())
		.chain(paint::mongo_collections())
		.chain(product::mongo_collections())
		.chain(role::mongo_collections())
		.chain(ticket::mongo_collections())
		.chain(user::mongo_collections())
		.chain(emote_moderation_request::mongo_collections())
		.chain(webhook_event::mongo_collections())
		.chain(cron_job::mongo_collections())
}

pub(super) async fn init_mongo(db: &mongodb::Database) -> anyhow::Result<()> {
	for collection in mongo_collections() {
		collection.init(db).await?;
	}

	Ok(())
}
