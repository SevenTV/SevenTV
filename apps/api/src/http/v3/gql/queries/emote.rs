use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, InputObject, Object, SimpleObject};
use fred::prelude::KeysInterface;
use shared::database::emote::EmoteId;
use shared::database::role::permissions::{EmotePermission, PermissionsExt};
use shared::old_types::image::ImageHost;
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::{EmoteFlagsModel, EmoteLifecycleModel, EmoteVersionState};
use shared::typesense::types::event::EventId;

use super::audit_log::AuditLog;
use super::report::Report;
use super::user::{UserPartial, UserSearchResult};
use crate::dataloader::emote::EmoteByIdLoaderExt;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::RateLimitGuard;
use crate::http::middleware::session::Session;
use crate::search::{search, sorted_results, SearchOptions};

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
		let host = ImageHost::from_image_set(&value.image_set, &global.config.api.cdn_origin);
		let state = EmoteVersionState::from_db(&value.flags);
		let listed = value.flags.contains(shared::database::emote::EmoteFlags::PublicListed);
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
				listed,
				host,
			}],
			state,
			listed,
			personal_use: value.flags.contains(shared::database::emote::EmoteFlags::ApprovedPersonal),
		}
	}
}

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/resolvers/emote/emote.go
#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl Emote {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.0.timestamp()
	}

	#[tracing::instrument(skip_all, name = "Emote::owner")]
	async fn owner(&self, ctx: &Context<'_>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		Ok(global
			.user_loader
			.load_fast(global, self.owner_id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.filter(|u| session.can_view(u))
			.map(|u| UserPartial::from_db(global, u))
			.unwrap_or_else(UserPartial::deleted_user))
	}

	#[graphql(guard = "RateLimitGuard::search(1)")]
	#[tracing::instrument(skip_all, name = "Emote::channels")]
	async fn channels(
		&self,
		ctx: &Context<'_>,
		#[graphql(validator(maximum = 10))] page: Option<u32>,
		#[graphql(validator(maximum = 100))] limit: Option<u32>,
	) -> Result<UserSearchResult, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		let options = SearchOptions::builder()
			.query("*".to_owned())
			.filter_by(format!("emotes: {}", self.id.0))
			.sort_by(vec!["role_hoist_rank:desc".to_owned()])
			.page(page)
			.per_page(limit.unwrap_or(30))
			.build();

		let result = search::<shared::typesense::types::user::User>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to search")
			})?;

		let users = global
			.user_loader
			.load_fast_many(global, result.hits.iter().copied())
			.await
			.map_err(|()| {
				tracing::error!("failed to load users");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load users")
			})?;

		Ok(UserSearchResult {
			total: result.found as u32,
			items: sorted_results(result.hits, users)
				.into_iter()
				.filter(|u| session.can_view(u))
				.map(|u| UserPartial::from_db(global, u))
				.collect(),
		})
	}

	async fn common_names(&self) -> Vec<EmoteCommonName> {
		// won't be implemented
		vec![]
	}

	#[tracing::instrument(skip_all, name = "Emote::trending")]
	async fn trending<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<u32>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let value: Option<String> = global.redis.get("emote_stats:trending_day").await.map_err(|err| {
			tracing::error!(error = %err, "failed to get trending emote stats");
			ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to get trending emote stats")
		})?;

		let Some(value) = value else {
			return Ok(None);
		};

		let values: Vec<EmoteId> = serde_json::from_str(&value).map_err(|err| {
			tracing::error!(error = %err, "failed to parse trending emote stats");
			ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to parse trending emote stats")
		})?;

		Ok(values.into_iter().position(|e| e == self.id.id()).map(|p| p as u32 + 1))
	}

	#[graphql(guard = "RateLimitGuard::search(1)")]
	#[tracing::instrument(skip_all, name = "Emote::activity")]
	async fn activity<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(maximum = 300))] limit: Option<u32>,
	) -> Result<Vec<AuditLog>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let options = SearchOptions::builder()
			.query("*".to_owned())
			.filter_by(format!("target_id: {}", EventId::Emote(self.id.id())))
			.sort_by(vec!["created_at:desc".to_owned()])
			.page(None)
			.per_page(limit.unwrap_or(20))
			.build();

		let result = search::<shared::typesense::types::event::Event>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to search")
			})?;

		let events = global
			.event_by_id_loader
			.load_many(result.hits.iter().copied())
			.await
			.map_err(|()| {
				tracing::error!("failed to load event");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load event")
			})?;

		Ok(sorted_results(result.hits, events)
			.into_iter()
			.filter_map(AuditLog::from_db)
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

	#[tracing::instrument(skip_all, name = "EmotePartial::owner")]
	async fn owner(&self, ctx: &Context<'_>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(global
			.user_loader
			.load_fast(global, self.owner_id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
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
	#[graphql(validator(max_length = 32))]
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
	#[tracing::instrument(skip_all, name = "EmotesQuery::emote")]
	async fn emote<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<Option<Emote>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote = global
			.emote_by_id_loader
			.load_exclude_deleted(id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote"))?;

		Ok(emote.map(|e| Emote::from_db(global, e)))
	}

	#[graphql(name = "emotesByID")]
	#[tracing::instrument(skip_all, name = "EmotesQuery::emotes_by_id")]
	async fn emotes_by_id<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(max_items = 100))] list: Vec<GqlObjectId>,
	) -> Result<Vec<EmotePartial>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emotes = global
			.emote_by_id_loader
			.load_many_exclude_deleted(list.into_iter().map(|i| i.id()))
			.await
			.map_err(|()| {
				tracing::error!("failed to load emotes");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emotes")
			})?;

		Ok(emotes.into_values().map(|e| Emote::from_db(global, e).into()).collect())
	}

	#[graphql(guard = "RateLimitGuard::search(1)")]
	#[tracing::instrument(skip_all, name = "EmotesQuery::emotes")]
	async fn emotes(
		&self,
		ctx: &Context<'_>,
		#[graphql(validator(max_length = 100))] query: String,
		#[graphql(validator(maximum = 100))] page: Option<u32>,
		#[graphql(validator(maximum = 100))] limit: Option<u32>,
		filter: Option<EmoteSearchFilter>,
		sort: Option<EmoteSearchSort>,
	) -> Result<EmoteSearchResult, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		let limit = limit.unwrap_or(30);
		let page = page.unwrap_or_default().max(1);

		// This filters out deleted & merged emotes
		let mut filters = vec!["deleted: false".to_owned()];

		if !session.has(EmotePermission::ViewUnlisted) {
			filters.push("flag_public_listed: true".to_owned());
			filters.push("flag_private: false".to_owned());
		}

		let mut query_by = vec!["default_name".to_owned()];
		let mut prefix = vec!["true".to_owned()];
		let mut query_by_weights = vec![4];

		let mut sort_by = vec!["_text_match(buckets: 10):desc".to_owned()];

		if let Some(filter) = &filter {
			if let Some(true) = filter.animated {
				filters.push("flag_animated: true".to_string());
			}

			if let Some(true) = filter.zero_width {
				filters.push("flag_default_zero_width: true".to_string());
			}

			if let Some(true) = filter.personal_use {
				filters.push("flag_approved_personal: true".to_string());
			}

			if let Some(true) = filter.exact_match {
				if !query.is_empty() {
					let sanitized = query.replace('`', "");
					filters.push(format!("default_name: `{}`", sanitized.trim_end_matches('\\')));
				}
			}

			let order = match sort.map(|s| s.order).unwrap_or(EmoteSearchSortOrder::Descending) {
				EmoteSearchSortOrder::Ascending => "asc",
				EmoteSearchSortOrder::Descending => "desc",
			};

			match filter.category {
				None | Some(EmoteSearchCategory::Top) => {
					sort_by.push(format!("score_top_all_time:{order}"));
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
					return Err(ApiError::not_implemented(
						ApiErrorCode::BadRequest,
						"global emote search is not implemented",
					));
				}
			}
		}

		if filter.as_ref().is_some_and(|f| !f.ignore_tags.unwrap_or_default()) {
			query_by.push("tags".to_owned());
			prefix.push("false".to_owned());
			query_by_weights.push(1);
		}

		let options = SearchOptions::builder()
			.query(query.clone())
			.page(page)
			.per_page(limit)
			.query_by(query_by)
			.filter_by(filters.join(" && "))
			.sort_by(sort_by)
			.query_by_weights(query_by_weights)
			.prefix(prefix)
			.prioritize_exact_match(true)
			.prioritize_token_position(true)
			.prioritize_num_matching_fields(false)
			.exaustive(true)
			.build();

		let result = search::<shared::typesense::types::emote::Emote>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to search")
			})?;

		let emotes = global
			.emote_by_id_loader
			.load_many_exclude_deleted(result.hits.iter().copied())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emotes"))?;

		Ok(EmoteSearchResult {
			count: result.found as u32,
			max_page: (result.found as u32 / limit + 1).min(100),
			items: sorted_results(result.hits, emotes)
				.into_iter()
				.map(|e| Emote::from_db(global, e))
				.collect(),
		})
	}
}
