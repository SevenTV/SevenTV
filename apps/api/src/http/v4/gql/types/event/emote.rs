use shared::database::emote::EmoteId;
use shared::database::stored_event::StoredEventEmoteData;
use shared::database::user::UserId;

use crate::http::v4::gql::types::EmoteFlags;

#[derive(async_graphql::Union)]
pub enum EventEmoteData {
	Upload(EventEmoteDataUpload),
	Process(EventEmoteDataProcess),
	ChangeName(EventEmoteDataChangeName),
	Merge(EventEmoteDataMerge),
	ChangeOwner(EventEmoteDataChangeOwner),
	ChangeTags(EventEmoteDataChangeTags),
	ChangeFlags(EventEmoteDataChangeFlags),
	Delete(EventEmoteDataDelete),
}

impl From<StoredEventEmoteData> for EventEmoteData {
	fn from(value: StoredEventEmoteData) -> Self {
		match value {
			StoredEventEmoteData::Upload => Self::Upload(EventEmoteDataUpload::default()),
			StoredEventEmoteData::Process { event } => Self::Process(EventEmoteDataProcess { event: event.into() }),
			StoredEventEmoteData::ChangeName { old, new } => Self::ChangeName(EventEmoteDataChangeName { old, new }),
			StoredEventEmoteData::Merge { new_emote_id } => Self::Merge(EventEmoteDataMerge { new_emote_id }),
			StoredEventEmoteData::ChangeOwner { old, new } => Self::ChangeOwner(EventEmoteDataChangeOwner { old, new }),
			StoredEventEmoteData::ChangeTags { old, new } => Self::ChangeTags(EventEmoteDataChangeTags { old, new }),
			StoredEventEmoteData::ChangeFlags { old, new } => Self::ChangeFlags(EventEmoteDataChangeFlags {
				old: old.into(),
				new: new.into(),
			}),
			StoredEventEmoteData::Delete => Self::Delete(EventEmoteDataDelete::default()),
		}
	}
}

#[derive(async_graphql::SimpleObject, Default)]
pub struct EventEmoteDataUpload {
	pub noop: bool,
}

#[derive(async_graphql::SimpleObject)]
pub struct EventEmoteDataProcess {
	pub event: ImageProcessorEvent,
}

#[derive(async_graphql::Enum, Copy, Clone, PartialEq, Eq, Debug)]
pub enum ImageProcessorEvent {
	Success,
	Fail,
	Cancel,
	Start,
}

impl From<shared::database::stored_event::ImageProcessorEvent> for ImageProcessorEvent {
	fn from(value: shared::database::stored_event::ImageProcessorEvent) -> Self {
		match value {
			shared::database::stored_event::ImageProcessorEvent::Success => Self::Success,
			shared::database::stored_event::ImageProcessorEvent::Fail { .. } => Self::Fail,
			shared::database::stored_event::ImageProcessorEvent::Cancel => Self::Cancel,
			shared::database::stored_event::ImageProcessorEvent::Start => Self::Start,
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct EventEmoteDataChangeName {
	pub old: String,
	pub new: String,
}

#[derive(async_graphql::SimpleObject)]
pub struct EventEmoteDataMerge {
	pub new_emote_id: EmoteId,
}

#[derive(async_graphql::SimpleObject)]
pub struct EventEmoteDataChangeOwner {
	pub old: UserId,
	pub new: UserId,
}

#[derive(async_graphql::SimpleObject)]
pub struct EventEmoteDataChangeTags {
	pub old: Vec<String>,
	pub new: Vec<String>,
}

#[derive(async_graphql::SimpleObject)]
pub struct EventEmoteDataChangeFlags {
	pub old: EmoteFlags,
	pub new: EmoteFlags,
}

#[derive(async_graphql::SimpleObject, Default)]
pub struct EventEmoteDataDelete {
	pub noop: bool,
}
