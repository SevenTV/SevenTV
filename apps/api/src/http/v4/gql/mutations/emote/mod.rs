use std::sync::Arc;

use async_graphql::Context;
use shared::database::emote::{EmoteFlags, EmoteId};

use crate::dataloader::emote::EmoteByIdLoaderExt;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

mod batch_operation;
mod operation;

#[derive(async_graphql::InputObject)]
pub struct EmoteFlagsInput {
	pub public_listed: Option<bool>,
	pub private: Option<bool>,
	pub nsfw: Option<bool>,
	pub default_zero_width: Option<bool>,
	pub approved_personal: Option<bool>,
	pub denied_personal: Option<bool>,
	pub animated: Option<bool>,
}

impl EmoteFlagsInput {
	fn apply_to(&self, mut flags: EmoteFlags) -> EmoteFlags {
		if let Some(public_listed) = self.public_listed {
			if public_listed {
				flags |= EmoteFlags::PublicListed;
			} else {
				flags &= !EmoteFlags::PublicListed;
			}
		}

		if let Some(private) = self.private {
			if private {
				flags |= EmoteFlags::Private;
			} else {
				flags &= !EmoteFlags::Private;
			}
		}

		if let Some(nsfw) = self.nsfw {
			if nsfw {
				flags |= EmoteFlags::Nsfw;
			} else {
				flags &= !EmoteFlags::Nsfw;
			}
		}

		if let Some(default_zero_width) = self.default_zero_width {
			if default_zero_width {
				flags |= EmoteFlags::DefaultZeroWidth;
			} else {
				flags &= !EmoteFlags::DefaultZeroWidth;
			}
		}

		if let Some(approved_personal) = self.approved_personal {
			if approved_personal {
				flags |= EmoteFlags::ApprovedPersonal;
			} else {
				flags &= !EmoteFlags::ApprovedPersonal;
			}
		}

		if let Some(denied_personal) = self.denied_personal {
			if denied_personal {
				flags |= EmoteFlags::DeniedPersonal;
			} else {
				flags &= !EmoteFlags::DeniedPersonal;
			}
		}

		if let Some(animated) = self.animated {
			if animated {
				flags |= EmoteFlags::Animated;
			} else {
				flags &= !EmoteFlags::Animated;
			}
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

		Ok(operation::EmoteOperation { emote })
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
