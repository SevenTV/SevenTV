#![allow(unused_imports, dead_code)]

mod automod;
mod badge;
mod emote;
mod emote_set;
mod file;
mod global;
mod json_string;
mod page;
mod paint;
mod product;
mod role;
mod ticket;
mod user;

pub use self::automod::*;
pub use self::badge::*;
pub use self::emote::*;
pub use self::emote_set::*;
pub use self::file::*;
pub use self::global::*;
pub use self::page::*;
pub use self::paint::*;
pub use self::product::*;
pub use self::role::*;
pub use self::ticket::*;
pub use self::user::*;

pub trait Collection {
	const NAME: &'static str;
}
