mod user;
mod cosmetic;
mod emote;
mod image;
mod emote_set;

pub use user::*;
pub use cosmetic::*;
pub use emote::*;
pub use image::*;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}