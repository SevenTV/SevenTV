use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, InputObject, Object, SimpleObject};
use hyper::StatusCode;
use shared::database::emote::EmoteId;
use shared::database::user::UserId;
use shared::old_types::image::ImageHost;
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::EmoteFlagsModel;

use super::audit_log::AuditLog;
use super::report::Report;
use super::user::{UserPartial, UserSearchResult};
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::types::{EmoteLifecycleModel, EmoteVersionState};
use crate::utils::{search, SearchOptions};

#[derive(Default)]
pub struct EmotesQuery;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/emotes.gql

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct Emote {
	id: GqlObjectId,
	name: String,
	flags: EmoteFlagsModel,
	lifecycle: EmoteLifecycleModel,
	tags: Vec<String>,
	animated: bool,
	// created_at
	owner_id: GqlObjectId,
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
	pub fn from_db(global: &Arc<Global>, value: shared::database::emote::Emote) -> Self {
		let host = ImageHost::from_image_set(&value.image_set, &global.config().api.cdn_origin);
		let state = EmoteVersionState::from_db(&value.flags);
		let listed = value.flags.contains(shared::database::emote::EmoteFlags::PublicListed);
		let lifecycle = if value.merged.is_some() {
			EmoteLifecycleModel::Deleted
		} else if value.image_set.input.is_pending() {
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
			animated: value.flags.contains(shared::database::emote::EmoteFlags::Animated),
			owner_id: value.owner_id.into(),
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
			personal_use: value.flags.contains(shared::database::emote::EmoteFlags::ApprovedPersonal),
		}
	}

	pub fn deleted_emote() -> Self {
		Self {
			id: GqlObjectId(EmoteId::nil().cast()),
			name: "*DeletedEmote".to_string(),
			lifecycle: EmoteLifecycleModel::Deleted,
			flags: EmoteFlagsModel::none(),
			tags: vec![],
			animated: false,
			owner_id: GqlObjectId(UserId::nil().cast()),
			host: ImageHost::default(),
			versions: vec![],
			state: vec![],
			listed: false,
			personal_use: false,
		}
	}
}

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/resolvers/emote/emote.go
#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl Emote {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.0.timestamp()
	}

	async fn owner(&self, ctx: &Context<'_>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(global
			.user_loader()
			.load_fast(global, self.owner_id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|u| UserPartial::from_db(global, u))
			.unwrap_or_else(UserPartial::deleted_user))
	}

	async fn channels(
		&self,
		ctx: &Context<'_>,
		page: Option<u32>,
		limit: Option<u32>,
	) -> Result<UserSearchResult, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		if limit.is_some_and(|l| l > 50) {
			return Err(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"limit cannot be greater than 100",
			));
		}

		if page.is_some_and(|p| p > 10) {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "page cannot be greater than 10"));
		}

		let options = SearchOptions::builder()
			.query("".to_owned())
			.query_by(vec!["id".to_owned()])
			.filter_by(format!("emotes: {}", self.id.0))
			.sort_by(vec!["role_rank:desc".to_owned()])
			.page(page)
			.per_page(limit)
			.build();

		let result = search::<shared::typesense::types::user::User>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		let users = global
			.user_loader()
			.load_fast_many(global, result.hits.iter().copied())
			.await
			.map_err(|()| {
				tracing::error!("failed to load users");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		Ok(UserSearchResult {
			total: result.found as u32,
			items: result
				.hits
				.into_iter()
				.filter_map(|id| users.get(&id).cloned())
				.map(|u| UserPartial::from_db(global, u))
				.collect(),
		})
	}

	async fn common_names(&self) -> Vec<EmoteCommonName> {
		// won't be implemented
		vec![]
	}

	async fn trending(&self) -> Result<Option<u32>, ApiError> {
		// TODO: implement with clickhouse
		// Err(ApiError::NOT_IMPLEMENTED)
		Ok(None)
	}

	async fn activity<'ctx>(&self, ctx: &Context<'ctx>, limit: Option<u32>) -> Result<Vec<AuditLog>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let options = SearchOptions::builder()
			.query("".to_owned())
			.query_by(vec!["id".to_owned()])
			.filter_by(format!("emotes: {}", self.id.0))
			.sort_by(vec!["updated_at:desc".to_owned()])
			.page(None)
			.per_page(limit)
			.build();

		let result = search::<shared::typesense::types::audit_log::AuditLog>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		let logs = global
			.audit_log_by_id_loader()
			.load_many(result.hits.iter().copied())
			.await
			.map_err(|()| {
				tracing::error!("failed to load audit logs");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		Ok(logs
			.into_values()
			.filter_map(|l| AuditLog::from_db(l, &HashMap::new()))
			.collect())
	}

	async fn reports(&self) -> Vec<Report> {
		// won't be implemented
		vec![]
	}
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmotePartial {
	id: GqlObjectId,
	name: String,
	flags: EmoteFlagsModel,
	lifecycle: EmoteLifecycleModel,
	tags: Vec<String>,
	animated: bool,
	// created_at
	owner_id: GqlObjectId,
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
		self.id.0.timestamp()
	}

	async fn owner(&self, ctx: &Context<'_>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(global
			.user_loader()
			.load_fast(global, self.owner_id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|u| UserPartial::from_db(global, u))
			.unwrap_or_else(UserPartial::deleted_user))
	}
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmoteVersion {
	id: GqlObjectId,
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
		self.id.0.timestamp()
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
	pub count: u32,
	pub max_page: u32,
	pub items: Vec<Emote>,
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmotesQuery {
	async fn emote<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<Option<Emote>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| {
			tracing::error!("failed to get global from context");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let emote = global
			.emote_by_id_loader()
			.load(id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(emote.map(|e| Emote::from_db(global, e)))
	}

	#[graphql(name = "emotesByID")]
	async fn emotes_by_id<'ctx>(&self, ctx: &Context<'ctx>, list: Vec<GqlObjectId>) -> Result<Vec<EmotePartial>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		if list.len() > 100 {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "list too large"));
		}

		let emote = global
			.emote_by_id_loader()
			.load_many(list.into_iter().map(|i| i.id()))
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(emote.into_iter().map(|(_, e)| Emote::from_db(global, e).into()).collect())
	}

	async fn emotes(
		&self,
		ctx: &Context<'_>,
		query: String,
		page: Option<u32>,
		limit: Option<u32>,
		filter: Option<EmoteSearchFilter>,
		sort: Option<EmoteSearchSort>,
	) -> Result<EmoteSearchResult, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		if limit.is_some_and(|l| l > 150) {
			return Err(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"limit cannot be greater than 150",
			));
		}

		let limit = limit.unwrap_or(30);
		if limit > 100 {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "limit cannot be greater than 100"));
		}

		if page.is_some_and(|p| p * limit > 10000) {
			return Err(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"cannot request more than 10000 emotes at once",
			));
		}

		let page = page.unwrap_or_default().max(1);

		let mut filters = Vec::new();

		let options = SearchOptions::builder().query(query.clone()).page(page).per_page(limit);

		let mut query_by = vec!["default_name".to_owned()];
		let mut query_by_weights = vec![4];

		let mut sort_by = vec!["_text_match(buckets: 10):desc".to_owned()];

		if let Some(filter) = &filter {
			if let Some(animated) = filter.animated {
				filters.push(format!("flag_animated: {animated}"));
			}

			if let Some(zero_width) = filter.zero_width {
				filters.push(format!("flag_default_zero_width: {zero_width}"));
			}

			if let Some(personal_use) = filter.personal_use {
				filters.push(format!("flag_approved_personal: {personal_use}"));
			}

			if let Some(true) = filter.exact_match {
				if !query.is_empty() {
					// TODO: prevent injection
					filters.push(format!("default_name: {query}"));
				}
			}

			let order = match sort.map(|s| s.order).unwrap_or(EmoteSearchSortOrder::Descending) {
				EmoteSearchSortOrder::Ascending => "asc",
				EmoteSearchSortOrder::Descending => "desc",
			};

			match filter.category {
				None | Some(EmoteSearchCategory::Top) => {
					sort_by.push("score_top_all_time:desc".to_owned());
				}
				Some(EmoteSearchCategory::Featured) | Some(EmoteSearchCategory::TrendingDay) => {
					sort_by.push(format!("score_trending_day:{order}"));
					filters.push("score_trending_day:>0".to_owned());
				}
				Some(EmoteSearchCategory::TrendingWeek) => {
					sort_by.push(format!("score_trending_week:{order}"));
					filters.push("score_trending_week:>0".to_owned());
				}
				Some(EmoteSearchCategory::TrendingMonth) => {
					sort_by.push(format!("score_trending_month:{order}"));
					filters.push("score_trending_month:>0".to_owned());
				}
				Some(EmoteSearchCategory::New) => {
					sort_by.push(format!("created_at:{order}"));
				}
				Some(EmoteSearchCategory::Global) => {
					// TODO: implement
					return Err(ApiError::NOT_IMPLEMENTED);
				}
			}
		}

		if filter.as_ref().map_or(false, |f| !f.ignore_tags.unwrap_or_default()) {
			query_by.push("tags".to_owned());
			query_by_weights.push(1);
		}

		let options = options.query_by(query_by).filter_by(filters.join(" && ")).query_by_weights(query_by_weights).build();

		let result = search::<shared::typesense::types::emote::Emote>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		let emotes = global
			.emote_by_id_loader()
			.load_many(result.hits.iter().copied())
			.await
			.map_err(|()| {
				tracing::error!("failed to load emotes");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		Ok(EmoteSearchResult {
			count: result.found as u32,
			max_page: result.found as u32 / limit + 1,
			items: result
				.hits
				.into_iter()
				.filter_map(|id| emotes.get(&id).cloned())
				.map(|e| Emote::from_db(global, e))
				.collect(),
		})
	}
}
