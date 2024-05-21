#![allow(unused_imports, dead_code)]

mod activity;
mod automod;
mod badge;
mod emote;
mod emote_set;
mod global;
mod image_set;
mod json_string;
mod page;
mod paint;
mod product;
mod role;
mod ticket;
mod user;

pub use self::activity::*;
pub use self::automod::*;
pub use self::badge::*;
pub use self::emote::*;
pub use self::emote_set::*;
pub use self::global::*;
pub use self::image_set::*;
pub use self::json_string::*;
pub use self::page::*;
pub use self::paint::*;
pub use self::product::*;
pub use self::role::*;
pub use self::ticket::*;
pub use self::user::*;

pub trait Collection {
	const COLLECTION_NAME: &'static str;

	fn collection(db: &mongodb::Database) -> mongodb::Collection<Self>
	where
		Self: Sized,
	{
		db.collection(Self::COLLECTION_NAME)
	}
}
