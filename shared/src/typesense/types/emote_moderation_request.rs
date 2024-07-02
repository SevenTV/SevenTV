use chrono::Utc;

use super::{TypesenseCollection, TypesenseGenericCollection};
use crate::database::emote::EmoteId;
use crate::database::emote_moderation_request::{
	EmoteModerationRequestId, EmoteModerationRequestKind, EmoteModerationRequestStatus,
};
use crate::database::user::UserId;
use crate::database::{self};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "emote_moderation_requests")]
#[serde(deny_unknown_fields)]
pub struct EmoteModerationRequest {
	pub id: EmoteModerationRequestId,
	pub user_id: UserId,
	pub kind: EmoteModerationRequestKind,
	pub reason: Option<String>,
	pub emote_id: EmoteId,
	pub status: EmoteModerationRequestStatus,
	pub country_code: Option<String>,
	pub assigned_to: Vec<UserId>,
	#[typesense(default_sort)]
	pub priority: i32,
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::emote_moderation_request::EmoteModerationRequest> for EmoteModerationRequest {
	fn from(value: database::emote_moderation_request::EmoteModerationRequest) -> Self {
		Self {
			id: value.id,
			user_id: value.user_id,
			kind: value.kind,
			reason: value.reason,
			emote_id: value.emote_id,
			status: value.status,
			country_code: value.country_code,
			assigned_to: value.assigned_to,
			priority: value.priority,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<EmoteModerationRequest>()]
}
