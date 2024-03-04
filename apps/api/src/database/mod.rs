mod badge;
mod emote;
mod file;
mod page;
mod paint;
mod product;
mod role;
mod user;

pub use badge::*;
pub use emote::*;
pub use file::*;
pub use page::*;
pub use paint::*;
pub use product::*;
pub use role::*;
pub use user::*;

pub trait Table {
	const TABLE_NAME: &'static str;
}
