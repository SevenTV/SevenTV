use async_graphql::Context;
use shared::database::emote::EmoteId;
use shared::database::role::permissions::{EmotePermission, RateLimitResource};
use shared::database::user::UserId;

use super::EmoteFlagsInput;
use crate::http::error::ApiError;
use crate::http::guards::{PermissionGuard, RateLimitGuard};
use crate::http::v4::gql::types::Emote;
use crate::http::validators::EmoteNameValidator;

pub struct EmoteBatchOperation {
	pub emotes: Vec<shared::database::emote::Emote>,
}

#[async_graphql::Object]
impl EmoteBatchOperation {
	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn name(
		&self,
		ctx: &Context<'_>,
		#[graphql(validator(custom = "EmoteNameValidator"))] name: String,
	) -> Result<Vec<Emote>, ApiError> {
		todo!()
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn flags(&self, ctx: &Context<'_>, flags: EmoteFlagsInput) -> Result<Vec<Emote>, ApiError> {
		todo!()
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn owner(&self, ctx: &Context<'_>, owner_id: UserId) -> Result<Vec<Emote>, ApiError> {
		todo!()
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn tags(&self, ctx: &Context<'_>, tags: Vec<String>) -> Result<Vec<Emote>, ApiError> {
		todo!()
	}

	#[graphql(
		guard = "PermissionGuard::one(EmotePermission::Merge).and(RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1))"
	)]
	async fn merge(&self, ctx: &Context<'_>, with: EmoteId) -> Result<Vec<Emote>, ApiError> {
		todo!()
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn delete(&self, ctx: &Context<'_>, reason: Option<String>) -> Result<Vec<Emote>, ApiError> {
		todo!()
	}
}