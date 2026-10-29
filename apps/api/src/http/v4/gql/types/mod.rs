mod badge;
mod color;
mod editor;
mod emote;
mod emote_set;
mod entitlement;
mod event;
mod image_set;
mod paint;
mod permission;
mod product;
mod profile_picture;
mod redeem_code;
mod role;
mod search;
mod subscription;
mod ticket;
mod time;
mod user;

pub use badge::*;
pub use color::*;
pub use editor::*;
pub use emote::*;
pub use emote_set::*;
pub use entitlement::*;
pub use event::*;
pub use image_set::*;
pub use paint::*;
pub use permission::*;
pub use product::*;
pub use profile_picture::*;
pub use redeem_code::*;
pub use role::*;
pub use search::*;
pub use subscription::*;
pub use ticket::*;
pub use time::*;
pub use user::*;

#[derive(async_graphql::SimpleObject, Debug, Clone)]
pub struct EmoteSetCopyResult {
	pub emote_set: EmoteSet,
	pub copied: u32,
	pub entries: Vec<EmoteSetCopyEntry>,
}

#[derive(async_graphql::SimpleObject, Debug, Clone)]
pub struct EmoteSetCopyEntry {
	pub emote_id: EmoteId,
	pub alias: String,
	pub status: EmoteSetCopyStatus,
	pub reason: Option<String>,
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum EmoteSetCopyStatus {
    Copied,
    SkippedDuplicateId,
    SkippedDuplicateAlias,
    SkippedPrivate,
    SkippedUnavailable,
    SkippedCapacity,
    Failed,
}
