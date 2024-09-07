use super::emote::EmoteId;
use super::user::UserId;
use super::{MongoCollection, MongoGenericCollection};
use crate::database::Id;
use crate::typesense::types::impl_typesense_type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(i32)]
pub enum EmoteModerationRequestKind {
	PublicListing = 0,
	PersonalUse = 1,
}

impl_typesense_type!(EmoteModerationRequestKind, Int32);

#[derive(Debug, Clone, PartialEq, Eq, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(i32)]
pub enum EmoteModerationRequestStatus {
	Pending = 0,
	Approved = 1,
	Denied = 2,
}

impl_typesense_type!(EmoteModerationRequestStatus, Int32);

pub type EmoteModerationRequestId = Id<EmoteModerationRequest>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection, PartialEq, Eq)]
#[mongo(collection_name = "emote_moderation_requests")]
#[allow(clippy::duplicated_attributes)]
#[mongo(index(fields(kind = 1, user_id = 1, status = 1)))]
#[mongo(index(fields(kind = 1, emote_id = 1), unique))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct EmoteModerationRequest {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: EmoteModerationRequestId,
	pub user_id: UserId,
	pub kind: EmoteModerationRequestKind,
	pub reason: Option<String>,
	pub emote_id: EmoteId,
	pub status: EmoteModerationRequestStatus,
	pub country_code: Option<String>,
	pub assigned_to: Vec<UserId>,
	pub priority: i32,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	std::iter::once(MongoGenericCollection::new::<EmoteModerationRequest>())
}
