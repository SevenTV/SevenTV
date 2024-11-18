use async_graphql::Context;
use shared::database::emote::EmoteId;
use shared::database::role::permissions::{EmotePermission, RateLimitResource};
use shared::database::user::UserId;

use super::EmoteFlagsInput;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::{PermissionGuard, RateLimitGuard};
use crate::http::v4::gql::types::Emote;
use crate::http::validators::EmoteNameValidator;

pub struct EmoteBatchOperation {
	pub _emotes: Vec<shared::database::emote::Emote>,
}

#[async_graphql::Object]
impl EmoteBatchOperation {
	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn name(
		&self,
		_ctx: &Context<'_>,
		#[graphql(validator(custom = "EmoteNameValidator"))] _name: String,
	) -> Result<Vec<Emote>, ApiError> {
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn flags(&self, _ctx: &Context<'_>, _flags: EmoteFlagsInput) -> Result<Vec<Emote>, ApiError> {
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn owner(&self, _ctx: &Context<'_>, _owner_id: UserId) -> Result<Vec<Emote>, ApiError> {
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn tags(&self, _ctx: &Context<'_>, _tags: Vec<String>) -> Result<Vec<Emote>, ApiError> {
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}

	#[graphql(
		guard = "PermissionGuard::one(EmotePermission::Merge).and(RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1))"
	)]
	async fn merge(&self, _ctx: &Context<'_>, _with: EmoteId) -> Result<Vec<Emote>, ApiError> {
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn delete(&self, _ctx: &Context<'_>, _reason: Option<String>) -> Result<Vec<Emote>, ApiError> {
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}
}
