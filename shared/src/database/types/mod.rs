#![allow(unused_imports, dead_code)]

mod activity;
mod automod;
mod badge;
mod emote;
mod file;
mod global;
mod page;
mod paint;
mod product;
mod role;
mod ticket;
mod user;
mod json_string;

pub use self::activity::*;
pub use self::automod::*;
pub use self::badge::*;
pub use self::emote::*;
pub use self::file::*;
pub use self::global::*;
pub use self::page::*;
pub use self::paint::*;
pub use self::product::*;
pub use self::role::*;
pub use self::ticket::*;
pub use self::user::*;

pub trait Table {
	const TABLE_NAME: &'static str;
}
