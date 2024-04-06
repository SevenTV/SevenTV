mod cosmetic;
mod emote;
mod emote_set;
mod entitlement;
mod image;
mod message;
mod role;
mod user;
mod user_connection;
mod user_presence;

pub use cosmetic::*;
pub use emote::*;
pub use emote_set::*;
pub use entitlement::*;
pub use image::*;
pub use message::*;
pub use role::*;
pub use user::*;
pub use user_connection::*;
pub use user_presence::*;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
	value == &T::default()
}
