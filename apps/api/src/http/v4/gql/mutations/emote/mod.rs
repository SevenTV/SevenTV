use std::sync::Arc;

use async_graphql::Context;
use shared::database::emote::EmoteId;

use crate::dataloader::emote::EmoteByIdLoaderExt;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

mod batch_operation;
mod operation;

#[derive(async_graphql::InputObject)]
pub struct EmoteFlagsInput {
	pub public_listed: bool,
	pub private: bool,
	pub nsfw: bool,
	pub default_zero_width: bool,
	pub approved_personal: bool,
	pub denied_personal: bool,
	pub animated: bool,
}

impl Into<shared::database::emote::EmoteFlags> for EmoteFlagsInput {
	fn into(self) -> shared::database::emote::EmoteFlags {
		let mut flags = shared::database::emote::EmoteFlags::default();

		if self.public_listed {
			flags |= shared::database::emote::EmoteFlags::PublicListed;
		}

		if self.private {
			flags |= shared::database::emote::EmoteFlags::Private;
		}

		if self.nsfw {
			flags |= shared::database::emote::EmoteFlags::Nsfw;
		}

		if self.default_zero_width {
			flags |= shared::database::emote::EmoteFlags::DefaultZeroWidth;
		}

		if self.approved_personal {
			flags |= shared::database::emote::EmoteFlags::ApprovedPersonal;
		}

		if self.denied_personal {
			flags |= shared::database::emote::EmoteFlags::DeniedPersonal;
		}

		if self.animated {
			flags |= shared::database::emote::EmoteFlags::Animated;
		}

		flags
	}
}

#[derive(Default)]
pub struct EmoteMutation;

#[async_graphql::Object]
impl EmoteMutation {
	async fn emote<'ctx>(&self, ctx: &Context<'ctx>, id: EmoteId) -> Result<operation::EmoteOperation, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote = global
			.emote_by_id_loader
			.load_exclude_deleted(id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote not found"))?;

		Ok(operation::EmoteOperation { emote })
	}

	async fn emotes<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(min_items = 1, max_items = 50))] ids: Vec<EmoteId>,
	) -> Result<batch_operation::EmoteBatchOperation, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emotes = global
			.emote_by_id_loader
			.load_many_exclude_deleted(ids)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emotes"))?
			.into_values()
			.collect();

		Ok(batch_operation::EmoteBatchOperation { emotes })
	}
}
