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

impl From<EmoteFlagsInput> for shared::database::emote::EmoteFlags {
	fn from(val: EmoteFlagsInput) -> Self {
		let mut flags = shared::database::emote::EmoteFlags::default();

		if val.public_listed {
			flags |= shared::database::emote::EmoteFlags::PublicListed;
		}

		if val.private {
			flags |= shared::database::emote::EmoteFlags::Private;
		}

		if val.nsfw {
			flags |= shared::database::emote::EmoteFlags::Nsfw;
		}

		if val.default_zero_width {
			flags |= shared::database::emote::EmoteFlags::DefaultZeroWidth;
		}

		if val.approved_personal {
			flags |= shared::database::emote::EmoteFlags::ApprovedPersonal;
		}

		if val.denied_personal {
			flags |= shared::database::emote::EmoteFlags::DeniedPersonal;
		}

		if val.animated {
			flags |= shared::database::emote::EmoteFlags::Animated;
		}

		flags
	}
}

#[derive(Default)]
pub struct EmoteMutation;

#[async_graphql::Object]
impl EmoteMutation {
	#[tracing::instrument(skip_all, name = "EmoteMutation::emote")]
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

		Ok(operation::EmoteOperation { emote: emote })
	}

	#[tracing::instrument(skip_all, name = "EmoteMutation::emotes")]
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

		Ok(batch_operation::EmoteBatchOperation { _emotes: emotes })
	}
}
