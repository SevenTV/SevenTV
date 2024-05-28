use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object};
use shared::{
	database::{EmoteId, UserId},
	types::old::{EmoteFlagsModel, EmoteLifecycleModel, EmoteVersionState, ImageHost, ImageHostKind},
};

use crate::{global::Global, http::error::ApiError};

use super::{
	audit_logs::AuditLog,
	reports::Report,
	users::{UserPartial, UserSearchResult},
};

#[derive(Default)]
pub struct EmotesQuery;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/emotes.gql

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct Emote {
	pub id: EmoteId,
	pub name: String,
	pub flags: EmoteFlagsModel,
	pub lifecycle: EmoteLifecycleModel,
	pub tags: Vec<String>,
	pub animated: bool,
	// created_at
	pub owner_id: UserId,
	// owner

	// channels
	// common_names
	// trending
	pub host: ImageHost,
	pub versions: Vec<EmoteVersion>,
	// activity
	pub state: Vec<EmoteVersionState>,
	pub listed: bool,
	pub personal_use: bool,
	// reports
}

impl Emote {
	fn from_db(global: &Arc<Global>, value: shared::database::Emote) -> Self {
		let host = ImageHost::from_image_set(
			&value.image_set,
			&global.config().api.cdn_base_url,
			ImageHostKind::Emote,
			&value.id,
		);
		let state = value.flags.to_old_state();
		let listed = value.flags.contains(shared::database::EmoteFlags::PublicListed);
		let lifecycle = if value.image_set.input.is_pending() {
			EmoteLifecycleModel::Pending
		} else {
			EmoteLifecycleModel::Live
		};

		Self {
			id: value.id,
			name: value.default_name.clone(),
			flags: value.flags.into(),
			lifecycle,
			tags: value.tags,
			animated: value.animated,
			owner_id: value.owner_id.map(Into::into).unwrap_or_default(),
			host: host.clone(),
			versions: vec![EmoteVersion {
				id: value.id,
				name: value.default_name,
				description: String::new(),
				lifecycle,
				error: None,
				state: state.clone(),
				listed: listed,
				host,
			}],
			state,
			listed,
			personal_use: value.flags.contains(shared::database::EmoteFlags::ApprovedPersonal),
		}
	}
}

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/resolvers/emote/emote.go
#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl Emote {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.timestamp()
	}

	async fn owner(&self, ctx: &Context<'_>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		UserPartial::load_from_db(global, self.owner_id).await
	}

	async fn channels(&self, ctx: &Context<'_>, page: u32, limit: u32) -> Result<UserSearchResult, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn common_names(&self) -> Vec<EmoteCommonName> {
		// not implemented
		vec![]
	}

	async fn trending(&self) -> Result<u32, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn activity(&self) -> Result<Vec<AuditLog>, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn reports(&self) -> Vec<Report> {
		// not implemented
		vec![]
	}
}

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmoteVersion {
	id: EmoteId,
	name: String,
	description: String,
	// created_at
	host: ImageHost,
	lifecycle: EmoteLifecycleModel,
	error: Option<String>, // always None
	state: Vec<EmoteVersionState>,
	listed: bool,
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl EmoteVersion {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.timestamp()
	}
}

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EmoteCommonName {
	pub name: String,
	pub count: u32,
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmotesQuery {
	async fn emote<'ctx>(&self, ctx: &Context<'ctx>, id: EmoteId) -> Result<Option<Emote>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| {
			tracing::error!("failed to get global from context");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let emote = global
			.emote_by_id_loader()
			.load(id)
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(emote.map(|e| Emote::from_db(global, e)))
	}
}
