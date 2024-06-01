use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, InputObject, Object, SimpleObject};
use hyper::StatusCode;
use shared::old_types::{EmoteFlagsModel, ImageHost, ImageHostKind};

use super::audit_logs::AuditLog;
use super::reports::Report;
use super::users::{UserPartial, UserSearchResult};
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::gql::object_id::{EmoteObjectId, UserObjectId};
use crate::http::v3::types::{EmoteLifecycleModel, EmoteVersionState};

#[derive(Default)]
pub struct EmotesQuery;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/emotes.gql

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct Emote {
	id: EmoteObjectId,
	name: String,
	flags: EmoteFlagsModel,
	lifecycle: EmoteLifecycleModel,
	tags: Vec<String>,
	animated: bool,
	// created_at
	owner_id: UserObjectId,
	// owner

	// channels
	// common_names
	// trending
	host: ImageHost,
	versions: Vec<EmoteVersion>,
	// activity
	state: Vec<EmoteVersionState>,
	listed: bool,
	personal_use: bool,
	// reports
}

impl Emote {
	pub fn from_db(global: &Arc<Global>, value: shared::database::Emote) -> Self {
		let host = ImageHost::from_image_set(
			&value.image_set,
			&global.config().api.cdn_base_url,
			ImageHostKind::Emote,
			&value.id,
		);
		let state = EmoteVersionState::from_db(&value.flags);
		let listed = value.flags.contains(shared::database::EmoteFlags::PublicListed);
		let lifecycle = if value.image_set.input.is_pending() {
			EmoteLifecycleModel::Pending
		} else {
			EmoteLifecycleModel::Live
		};

		Self {
			id: value.id.into(),
			name: value.default_name.clone(),
			flags: value.flags.into(),
			lifecycle,
			tags: value.tags,
			animated: value.animated,
			owner_id: value.owner_id.map(Into::into).unwrap_or_default(),
			host: host.clone(),
			versions: vec![EmoteVersion {
				id: value.id.into(),
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
		UserPartial::load_from_db(global, *self.owner_id).await
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

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmotePartial {
	id: EmoteObjectId,
	name: String,
	flags: EmoteFlagsModel,
	lifecycle: EmoteLifecycleModel,
	tags: Vec<String>,
	animated: bool,
	// created_at
	owner_id: UserObjectId,
	// owner
	host: ImageHost,
	state: Vec<EmoteVersionState>,
	listed: bool,
}

impl From<Emote> for EmotePartial {
	fn from(value: Emote) -> Self {
		Self {
			id: value.id,
			name: value.name,
			flags: value.flags,
			lifecycle: value.lifecycle,
			tags: value.tags,
			animated: value.animated,
			owner_id: value.owner_id,
			host: value.host,
			state: value.state,
			listed: value.listed,
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl EmotePartial {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.timestamp()
	}

	async fn owner(&self, ctx: &Context<'_>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		UserPartial::load_from_db(global, *self.owner_id).await
	}
}

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmoteVersion {
	id: EmoteObjectId,
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

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EmoteCommonName {
	name: String,
	count: u32,
}

#[derive(Debug, Clone, Default, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EmoteSearchFilter {
	category: Option<EmoteSearchCategory>,
	case_sensitive: Option<bool>,
	exact_match: Option<bool>,
	ignore_tags: Option<bool>,
	animated: Option<bool>,
	zero_width: Option<bool>,
	authentic: Option<bool>,
	aspect_ratio: Option<String>,
	personal_use: Option<bool>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum EmoteSearchCategory {
	Top,
	TrendingDay,
	TrendingWeek,
	TrendingMonth,
	Featured,
	New,
	Global,
}

#[derive(Debug, Clone, Default, InputObject)]
#[graphql(name = "Sort", rename_fields = "snake_case")]
pub struct EmoteSearchSort {
	value: String,
	order: EmoteSearchSortOrder,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Enum)]
#[graphql(name = "SortOrder", rename_items = "SCREAMING_SNAKE_CASE")]
pub enum EmoteSearchSortOrder {
	#[default]
	Ascending,
	Descending,
}

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EmoteSearchResult {
	count: u32,
	max_page: u32,
	items: Vec<Emote>,
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmotesQuery {
	async fn emote<'ctx>(&self, ctx: &Context<'ctx>, id: EmoteObjectId) -> Result<Option<Emote>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| {
			tracing::error!("failed to get global from context");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let emote = global
			.emote_by_id_loader()
			.load(*id)
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(emote.map(|e| Emote::from_db(global, e)))
	}

	#[graphql(name = "emotesByID")]
	async fn emotes_by_id<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		list: Vec<EmoteObjectId>,
	) -> Result<Vec<EmotePartial>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		if list.len() > 1000 {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "list too large"));
		}

		let emote = global
			.emote_by_id_loader()
			.load_many(list.into_iter().map(|i| *i))
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(emote.into_iter().map(|(_, e)| Emote::from_db(global, e).into()).collect())
	}

	async fn emotes(
		&self,
		ctx: &Context<'_>,
		page: Option<u32>,
		limit: Option<u32>,
		filter: Option<EmoteSearchFilter>,
		sort: Option<EmoteSearchSort>,
	) -> Result<Vec<Emote>, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}
}
