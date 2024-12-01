use std::sync::Arc;

use async_graphql::Context;
use shared::database::emote::EmoteId;
use shared::database::stored_event::StoredEventEmoteData;
use shared::database::user::UserId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v4::gql::types::{Emote, EmoteFlags, User};

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
			StoredEventEmoteData::ChangeOwner { old, new } => Self::ChangeOwner(EventEmoteDataChangeOwner {
				old_id: old,
				new_id: new,
			}),
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
	/// Always false
	#[graphql(deprecation = true)]
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
	#[graphql(name = "oldName")]
	pub old: String,
	#[graphql(name = "newName")]
	pub new: String,
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EventEmoteDataMerge {
	pub new_emote_id: EmoteId,
}

#[async_graphql::ComplexObject]
impl EventEmoteDataMerge {
	#[tracing::instrument(skip_all, name = "EventEmoteDataMerge::new_emote")]
	async fn new_emote(&self, ctx: &Context<'_>) -> Result<Emote, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote = global
			.emote_by_id_loader
			.load(self.new_emote_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote not found"))?;

		Ok(Emote::from_db(emote, &global.config.api.cdn_origin))
	}
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EventEmoteDataChangeOwner {
	#[graphql(name = "oldOwnerId")]
	pub old_id: UserId,
	#[graphql(name = "newOwnerId")]
	pub new_id: UserId,
}

#[async_graphql::ComplexObject]
impl EventEmoteDataChangeOwner {
	#[tracing::instrument(skip_all, name = "EventEmoteDataChangeOwner::old_owner")]
	async fn old_owner(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_loader
			.load(global, self.old_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(user.map(Into::into))
	}

	#[tracing::instrument(skip_all, name = "EventEmoteDataChangeOwner::new_owner")]
	async fn new_owner(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_loader
			.load(global, self.new_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(user.map(Into::into))
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct EventEmoteDataChangeTags {
	#[graphql(name = "oldTags")]
	pub old: Vec<String>,
	#[graphql(name = "newTags")]
	pub new: Vec<String>,
}

#[derive(async_graphql::SimpleObject)]
pub struct EventEmoteDataChangeFlags {
	#[graphql(name = "oldFlags")]
	pub old: EmoteFlags,
	#[graphql(name = "newFlags")]
	pub new: EmoteFlags,
}

#[derive(async_graphql::SimpleObject, Default)]
pub struct EventEmoteDataDelete {
	/// Always false
	#[graphql(deprecation = true)]
	pub noop: bool,
}
