use std::sync::Arc;

use async_graphql::Context;
use shared::database::emote::EmoteId;
use shared::database::stored_event::StoredEventEmoteSetData;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v4::gql::types::Emote;

#[derive(async_graphql::Union)]
pub enum EventEmoteSetData {
	Create(EventEmoteSetDataCreate),
	ChangeName(EventEmoteSetDataChangeName),
	ChangeTags(EventEmoteSetDataChangeTags),
	ChangeCapacity(EventEmoteSetDataChangeCapacity),
	AddEmote(EventEmoteSetDataAddEmote),
	RemoveEmote(EventEmoteSetDataRemoveEmote),
	RenameEmote(EventEmoteSetDataRenameEmote),
	Delete(EventEmoteSetDataDelete),
}

impl From<StoredEventEmoteSetData> for EventEmoteSetData {
	fn from(value: StoredEventEmoteSetData) -> Self {
		match value {
			StoredEventEmoteSetData::Create => Self::Create(EventEmoteSetDataCreate::default()),
			StoredEventEmoteSetData::ChangeName { old, new } => Self::ChangeName(EventEmoteSetDataChangeName { old, new }),
			StoredEventEmoteSetData::ChangeTags { old, new } => Self::ChangeTags(EventEmoteSetDataChangeTags { old, new }),
			StoredEventEmoteSetData::ChangeCapacity { old, new } => {
				Self::ChangeCapacity(EventEmoteSetDataChangeCapacity { old, new })
			}
			StoredEventEmoteSetData::AddEmote { emote_id, alias } => {
				Self::AddEmote(EventEmoteSetDataAddEmote { emote_id, alias })
			}
			StoredEventEmoteSetData::RemoveEmote { emote_id } => {
				Self::RemoveEmote(EventEmoteSetDataRemoveEmote { emote_id })
			}
			StoredEventEmoteSetData::RenameEmote {
				emote_id,
				old_alias,
				new_alias,
			} => Self::RenameEmote(EventEmoteSetDataRenameEmote {
				emote_id,
				old_alias,
				new_alias,
			}),
			StoredEventEmoteSetData::Delete => Self::Delete(EventEmoteSetDataDelete::default()),
		}
	}
}

#[derive(async_graphql::SimpleObject, Default)]
pub struct EventEmoteSetDataCreate {
	/// Always false
	#[graphql(deprecation = true)]
	pub noop: bool,
}

#[derive(async_graphql::SimpleObject)]
pub struct EventEmoteSetDataChangeName {
	#[graphql(name = "oldName")]
	pub old: String,
	#[graphql(name = "newName")]
	pub new: String,
}

#[derive(async_graphql::SimpleObject)]
pub struct EventEmoteSetDataChangeTags {
	#[graphql(name = "oldTags")]
	pub old: Vec<String>,
	#[graphql(name = "newTags")]
	pub new: Vec<String>,
}

#[derive(async_graphql::SimpleObject)]
pub struct EventEmoteSetDataChangeCapacity {
	#[graphql(name = "oldCapacity")]
	pub old: Option<i32>,
	#[graphql(name = "newCapacity")]
	pub new: Option<i32>,
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EventEmoteSetDataAddEmote {
	#[graphql(name = "addedEmoteId")]
	pub emote_id: EmoteId,
	pub alias: String,
}

#[async_graphql::ComplexObject]
impl EventEmoteSetDataAddEmote {
	#[tracing::instrument(skip_all, name = "EventEmoteSetDataAddEmote::added_emote")]
	async fn added_emote(&self, ctx: &Context<'_>) -> Result<Option<Emote>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote = global
			.emote_by_id_loader
			.load(self.emote_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote"))?;

		Ok(emote.map(|e| Emote::from_db(e, &global.config.api.cdn_origin)))
	}
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EventEmoteSetDataRemoveEmote {
	#[graphql(name = "removedEmoteId")]
	pub emote_id: EmoteId,
}

#[async_graphql::ComplexObject]
impl EventEmoteSetDataRemoveEmote {
	#[tracing::instrument(skip_all, name = "EventEmoteSetDataRemoveEmote::removed_emote")]
	async fn removed_emote(&self, ctx: &Context<'_>) -> Result<Option<Emote>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote = global
			.emote_by_id_loader
			.load(self.emote_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote"))?;

		Ok(emote.map(|e| Emote::from_db(e, &global.config.api.cdn_origin)))
	}
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EventEmoteSetDataRenameEmote {
	#[graphql(name = "renamedEmoteId")]
	pub emote_id: EmoteId,
	pub old_alias: String,
	pub new_alias: String,
}

#[async_graphql::ComplexObject]
impl EventEmoteSetDataRenameEmote {
	#[tracing::instrument(skip_all, name = "EventEmoteSetDataRenameEmote::renamed_emote")]
	async fn renamed_emote(&self, ctx: &Context<'_>) -> Result<Option<Emote>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote = global
			.emote_by_id_loader
			.load(self.emote_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote"))?;

		Ok(emote.map(|e| Emote::from_db(e, &global.config.api.cdn_origin)))
	}
}

#[derive(async_graphql::SimpleObject, Default)]
pub struct EventEmoteSetDataDelete {
	/// Always false
	#[graphql(deprecation = true)]
	pub noop: bool,
}
