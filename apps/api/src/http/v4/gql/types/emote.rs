use std::sync::Arc;

use async_graphql::{ComplexObject, SimpleObject, Context};
use shared::database::{emote::EmoteId, user::UserId};

use crate::{global::Global, http::error::{ApiError, ApiErrorCode}};

use super::{Image, User};

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Emote {
	pub id: EmoteId,
	pub owner_id: UserId,
	pub default_name: String,
	pub tags: Vec<String>,
	pub images: Vec<Image>,
	pub flags: EmoteFlags,
	pub aspect_ratio: f64,
	pub attribution: Vec<EmoteAttribution>,
	pub scores: EmoteScores,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[ComplexObject]
impl Emote {
	pub async fn owner<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_loader
			.load(global, self.owner_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(user.map(Into::into))
	}
}

impl Emote {
	pub fn from_db(value: shared::database::emote::Emote, cdn_base_url: &url::Url) -> Self {
		Self {
			id: value.id,
			owner_id: value.owner_id,
			default_name: value.default_name,
			tags: value.tags,
			images: value
				.image_set
				.outputs
				.into_iter()
				.map(|o| Image::from_db(o, cdn_base_url))
				.collect(),
			flags: value.flags.into(),
			aspect_ratio: value.aspect_ratio,
			attribution: value.attribution.into_iter().map(Into::into).collect(),
			scores: value.scores.into(),
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		}
	}
}

#[derive(Debug, Clone, SimpleObject)]
pub struct EmoteFlags {
	pub public_listed: bool,
	pub private: bool,
	pub nsfw: bool,
	pub default_zero_width: bool,
	pub approved_personal: bool,
	pub denied_personal: bool,
	pub animated: bool,
}

impl From<shared::database::emote::EmoteFlags> for EmoteFlags {
	fn from(value: shared::database::emote::EmoteFlags) -> Self {
		Self {
			public_listed: value.contains(shared::database::emote::EmoteFlags::PublicListed),
			private: value.contains(shared::database::emote::EmoteFlags::Private),
			nsfw: value.contains(shared::database::emote::EmoteFlags::Nsfw),
			default_zero_width: value.contains(shared::database::emote::EmoteFlags::DefaultZeroWidth),
			approved_personal: value.contains(shared::database::emote::EmoteFlags::ApprovedPersonal),
			denied_personal: value.contains(shared::database::emote::EmoteFlags::DeniedPersonal),
			animated: value.contains(shared::database::emote::EmoteFlags::Animated),
		}
	}
}

#[derive(Debug, Clone, SimpleObject)]
pub struct EmoteScores {
	pub trending_day: i32,
	pub trending_week: i32,
	pub trending_month: i32,
	pub top_daily: i32,
	pub top_weekly: i32,
	pub top_monthly: i32,
	pub top_all_time: i32,
}

impl From<shared::database::emote::EmoteScores> for EmoteScores {
	fn from(value: shared::database::emote::EmoteScores) -> Self {
		Self {
			trending_day: value.trending_day,
			trending_week: value.trending_week,
			trending_month: value.trending_month,
			top_daily: value.top_daily,
			top_weekly: value.top_weekly,
			top_monthly: value.top_monthly,
			top_all_time: value.top_all_time,
		}
	}
}

#[derive(Debug, Clone, SimpleObject)]
pub struct EmoteAttribution {
	pub user_id: UserId,
	pub added_at: chrono::DateTime<chrono::Utc>,
}

impl From<shared::database::emote::EmoteAttribution> for EmoteAttribution {
	fn from(value: shared::database::emote::EmoteAttribution) -> Self {
		Self {
			user_id: value.user_id,
			added_at: value.added_at,
		}
	}
}
