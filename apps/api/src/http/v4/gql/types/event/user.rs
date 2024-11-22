use std::sync::Arc;

use async_graphql::Context;
use shared::database::badge::BadgeId;
use shared::database::emote_set::EmoteSetId;
use shared::database::paint::PaintId;
use shared::database::stored_event::StoredEventUserData;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v4::gql::types::{EmoteSet, Paint, Platform};

#[derive(async_graphql::Union)]
pub enum EventUserData {
	Create(EventUserDataCreate),
	ChangeActivePaint(EventUserDataChangeActivePaint),
	ChangeActiveBadge(EventUserDataChangeActiveBadge),
	ChangeActiveEmoteSet(EventUserDataChangeActiveEmoteSet),
	AddConnection(EventUserDataAddConnection),
	RemoveConnection(EventUserDataRemoveConnection),
	// AddEntitlement(EventUserDataAddEntitlement),
	// RemoveEntitlement(EventUserDataRemoveEntitlement),
	// Merge(EventUserDataMerge),
	Delete(EventUserDataDelete),
}

impl TryFrom<StoredEventUserData> for EventUserData {
	type Error = ();

	fn try_from(value: StoredEventUserData) -> Result<Self, Self::Error> {
		match value {
			StoredEventUserData::Create => Ok(Self::Create(EventUserDataCreate::default())),
			StoredEventUserData::ChangeActivePaint { old, new } => {
				Ok(Self::ChangeActivePaint(EventUserDataChangeActivePaint {
					old_id: old,
					new_id: new,
				}))
			}
			StoredEventUserData::ChangeActiveBadge { old, new } => {
				Ok(Self::ChangeActiveBadge(EventUserDataChangeActiveBadge {
					old_id: old,
					new_id: new,
				}))
			}
			StoredEventUserData::ChangeActiveEmoteSet { old, new } => {
				Ok(Self::ChangeActiveEmoteSet(EventUserDataChangeActiveEmoteSet {
					old_id: old,
					new_id: new,
				}))
			}
			StoredEventUserData::AddConnection { platform } => Ok(Self::AddConnection(EventUserDataAddConnection {
				platform: platform.into(),
			})),
			StoredEventUserData::RemoveConnection { platform } => {
				Ok(Self::RemoveConnection(EventUserDataRemoveConnection {
					platform: platform.into(),
				}))
			}
			StoredEventUserData::Delete => Ok(Self::Delete(EventUserDataDelete::default())),
			_ => Err(()),
		}
	}
}

#[derive(async_graphql::SimpleObject, Default)]
pub struct EventUserDataCreate {
	/// Always false
	#[graphql(deprecation = true)]
	pub noop: bool,
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EventUserDataChangeActivePaint {
	#[graphql(name = "oldPaintId")]
	pub old_id: Option<PaintId>,
	#[graphql(name = "newPaintId")]
	pub new_id: Option<PaintId>,
}

#[async_graphql::ComplexObject]
impl EventUserDataChangeActivePaint {
	async fn old_paint(&self, ctx: &Context<'_>) -> Result<Option<Paint>, ApiError> {
		let Some(old_id) = self.old_id else {
			return Ok(None);
		};

		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let paint = global
			.paint_by_id_loader
			.load(old_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load paint"))?;

		Ok(paint.map(|p| Paint::from_db(p, &global.config.api.cdn_origin)))
	}

	async fn new_paint(&self, ctx: &Context<'_>) -> Result<Option<Paint>, ApiError> {
		let Some(old_id) = self.new_id else {
			return Ok(None);
		};

		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let paint = global
			.paint_by_id_loader
			.load(old_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load paint"))?;

		Ok(paint.map(|p| Paint::from_db(p, &global.config.api.cdn_origin)))
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct EventUserDataChangeActiveBadge {
	#[graphql(name = "oldBadgeId")]
	pub old_id: Option<BadgeId>,
	#[graphql(name = "newBadgeId")]
	pub new_id: Option<BadgeId>,
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct EventUserDataChangeActiveEmoteSet {
	#[graphql(name = "oldEmoteSetId")]
	pub old_id: Option<EmoteSetId>,
	#[graphql(name = "newEmoteSetId")]
	pub new_id: Option<EmoteSetId>,
}

#[async_graphql::ComplexObject]
impl EventUserDataChangeActiveEmoteSet {
	async fn old_emote_set(&self, ctx: &Context<'_>) -> Result<Option<EmoteSet>, ApiError> {
		let Some(old_id) = self.old_id else {
			return Ok(None);
		};

		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(old_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?;

		Ok(emote_set.map(Into::into))
	}

	async fn new_emote_set(&self, ctx: &Context<'_>) -> Result<Option<EmoteSet>, ApiError> {
		let Some(new_id) = self.new_id else {
			return Ok(None);
		};

		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(new_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?;

		Ok(emote_set.map(Into::into))
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct EventUserDataAddConnection {
	#[graphql(name = "addedPlatform")]
	pub platform: Platform,
}

#[derive(async_graphql::SimpleObject)]
pub struct EventUserDataRemoveConnection {
	#[graphql(name = "removedPlatform")]
	pub platform: Platform,
}

// #[derive(async_graphql::SimpleObject)]
// pub struct EventUserDataAddEntitlement {
// 	#[graphql(name = "addedTarget")]
// 	pub target: EntitlementEdgeKind,
// }

// #[derive(async_graphql::SimpleObject)]
// pub struct EventUserDataRemoveEntitlement {
// 	#[graphql(name = "removedTarget")]
// 	pub target: EntitlementEdgeKind,
// }

// #[derive(async_graphql::SimpleObject)]
// pub struct EventUserDataMerge {
// 	pub source_id: UserId,
// 	pub connections: Vec<UserConnection>,
// }

#[derive(async_graphql::SimpleObject, Default)]
pub struct EventUserDataDelete {
	/// Always false
	#[graphql(deprecation = true)]
	pub noop: bool,
}
