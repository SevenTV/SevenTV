pub mod audit_log;
pub mod automod;
pub mod badge;
pub mod duration;
pub mod emote;
pub mod emote_moderation_request;
pub mod emote_set;
pub mod entitlement;
pub mod global;
pub mod image_set;
pub mod json_string;
pub mod page;
pub mod paint;
pub mod product;
pub mod role;
pub mod ticket;
pub mod user;

pub trait Collection: Send + Sync {
	const COLLECTION_NAME: &'static str;

	fn collection(db: &mongodb::Database) -> mongodb::Collection<Self>
	where
		Self: Sized,
	{
		db.collection(Self::COLLECTION_NAME)
	}

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![]
	}
}

struct GenericCollection {
	name: &'static str,
	indexes: Vec<mongodb::IndexModel>,
}

impl GenericCollection {
	fn new<C: Collection>() -> Self {
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

fn collections() -> impl IntoIterator<Item = GenericCollection> {
	std::iter::empty()
		.chain(audit_log::collections())
		.chain(automod::collections())
		.chain(badge::collections())
		.chain(emote::collections())
		.chain(emote_set::collections())
		.chain(entitlement::collections())
		.chain(global::collections())
		.chain(page::collections())
		.chain(paint::collections())
		.chain(product::collections())
		.chain(role::collections())
		.chain(ticket::collections())
		.chain(user::collections())
		.chain(emote_moderation_request::collections())
}

pub(super) async fn init_database(db: &mongodb::Database) -> anyhow::Result<()> {
	for collection in collections() {
		collection.init(db).await?;
	}

	Ok(())
}
