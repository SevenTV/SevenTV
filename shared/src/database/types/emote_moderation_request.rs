use mongodb::options::IndexOptions;

use super::emote::EmoteId;
use super::user::UserId;
use super::{Collection, GenericCollection};
use crate::database::Id;

#[derive(Debug, Clone, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(i32)]
pub enum EmoteModerationRequestKind {
	PublicListing = 0,
	PersonalUse = 1,
}

#[derive(Debug, Clone, PartialEq, Eq, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(i32)]
pub enum EmoteModerationRequestStatus {
	Pending = 0,
	Approved = 1,
	Denied = 2,
}

pub type EmoteModerationRequestId = Id<EmoteModerationRequest>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmoteModerationRequest {
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
}

impl Collection for EmoteModerationRequest {
	const COLLECTION_NAME: &'static str = "emote_moderation_requests";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"kind": 1,
					"user_id": 1,
					"status": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"kind": 1,
					"emote_id": 1,
				})
				.options(IndexOptions::builder().unique(true).build())
				.build(),
		]
	}
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	std::iter::once(GenericCollection::new::<EmoteModerationRequest>())
}
