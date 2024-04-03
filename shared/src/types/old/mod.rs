mod user;
mod cosmetic;
mod emote;
mod image;
mod emote_set;
mod entitlement;
mod message;
mod role;
mod user_connection;
mod user_presence;

pub use user::*;
pub use cosmetic::*;
pub use emote::*;
pub use image::*;
pub use emote_set::*;
pub use entitlement::*;
pub use message::*;
pub use role::*;
pub use user_connection::*;
pub use user_presence::*;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}