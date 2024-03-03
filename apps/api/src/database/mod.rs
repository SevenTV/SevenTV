pub mod badge;
pub mod emote;
pub mod file;
pub mod page;
pub mod paint;
pub mod role;
pub mod user;

pub trait Table {
	const TABLE_NAME: &'static str;
}
