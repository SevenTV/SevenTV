use std::sync::Arc;

use async_graphql::Context;
use shared::database::badge::BadgeId;
use shared::database::emote_set::EmoteSetId;
use shared::database::paint::PaintId;
use shared::database::user::profile_picture::UserProfilePictureId;
use shared::database::user::FullUser;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v4::gql::types::{EmoteSet, Paint, UserProfilePicture};

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct UserStyle {
	pub active_badge_id: Option<BadgeId>,
	pub active_paint_id: Option<PaintId>,
	pub active_emote_set_id: Option<EmoteSetId>,
	pub active_profile_picture_id: Option<UserProfilePictureId>,
	pub active_profile_picture: Option<UserProfilePicture>,
	pub pending_profile_picture_id: Option<UserProfilePictureId>,
}

impl UserStyle {
	pub fn from_user(global: &Arc<Global>, user: &FullUser) -> Self {
		UserStyle {
			active_badge_id: user.style.active_badge_id,
			active_paint_id: user.style.active_paint_id,
			active_emote_set_id: user.style.active_emote_set_id,
			active_profile_picture_id: user.style.active_profile_picture,
			active_profile_picture: user
				.active_profile_picture
				.as_ref()
				.map(|p| UserProfilePicture::from_db(p.clone(), &global.config.api.cdn_origin)),
			pending_profile_picture_id: user.style.pending_profile_picture,
		}
	}
}

#[async_graphql::ComplexObject]
impl UserStyle {
	async fn active_paint(&self, ctx: &Context<'_>) -> Result<Option<Paint>, ApiError> {
		let Some(active_paint_id) = self.active_paint_id else {
			return Ok(None);
		};

		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let paint = global
			.paint_by_id_loader
			.load(active_paint_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load active paint"))?;

		Ok(paint.map(|p| Paint::from_db(p, &global.config.api.cdn_origin)))
	}

	async fn active_emote_set(&self, ctx: &Context<'_>) -> Result<Option<EmoteSet>, ApiError> {
		let Some(active_emote_set_id) = self.active_emote_set_id else {
			return Ok(None);
		};

		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(active_emote_set_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load active emote set"))?;

		Ok(emote_set.map(Into::into))
	}
}
