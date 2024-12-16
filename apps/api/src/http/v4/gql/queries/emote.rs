use std::fmt::Display;
use std::sync::Arc;

use async_graphql::{Context, Enum, InputObject, Object};
use itertools::Itertools;
use shared::database::emote::EmoteId;
use shared::database::role::permissions::{EmotePermission, PermissionsExt};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::RateLimitGuard;
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::{Emote, SearchResult};
use crate::search::{search, sorted_results, SearchOptions};

#[derive(Default)]
pub struct EmoteQuery;

#[derive(Debug, Clone, InputObject)]
struct Tags {
	#[graphql(validator(max_items = 10))]
	tags: Vec<String>,
	#[graphql(name = "match")]
	match_: TagsMatch,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
enum TagsMatch {
	All,
	Any,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
enum SortBy {
	TrendingDaily,
	TrendingWeekly,
	TrendingMonthly,
	TopDaily,
	TopWeekly,
	TopMonthly,
	TopAllTime,
	NameAlphabetical,
	UploadDate,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
enum SortOrder {
	Ascending,
	Descending,
}

impl Display for SortOrder {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Ascending => write!(f, "asc"),
			Self::Descending => write!(f, "desc"),
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, InputObject)]
struct Sort {
	sort_by: SortBy,
	order: SortOrder,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, InputObject)]
struct Filters {
	animated: Option<bool>,
	default_zero_width: Option<bool>,
	nsfw: Option<bool>,
	approved_personal: Option<bool>,
	/// defaults to false when unset
	exact_match: Option<bool>,
}

#[Object]
impl EmoteQuery {
	#[tracing::instrument(skip_all, name = "EmoteQuery::emote")]
	async fn emote(&self, ctx: &Context<'_>, id: EmoteId) -> Result<Option<Emote>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		let Some(emote) = global
			.emote_by_id_loader
			.load(id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote"))?
		else {
			return Ok(None);
		};

		if !session.has(EmotePermission::ViewUnlisted) && (emote.deleted || emote.merged.is_some()) {
			return Ok(None);
		}

		Ok(Some(Emote::from_db(emote, &global.config.api.cdn_origin)))
	}

	#[allow(clippy::too_many_arguments)]
	#[graphql(guard = "RateLimitGuard::search(1)")]
	#[tracing::instrument(skip_all, name = "EmoteQuery::search")]
	async fn search(
		&self,
		ctx: &Context<'_>,
		#[graphql(validator(max_length = 100))] query: Option<String>,
		tags: Option<Tags>,
		sort: Sort,
		filters: Option<Filters>,
		#[graphql(validator(maximum = 100))] page: Option<u32>,
		#[graphql(validator(minimum = 1, maximum = 250))] per_page: Option<u32>,
	) -> Result<SearchResult<Emote>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		let per_page = per_page.unwrap_or(30);
		let page = page.unwrap_or_default().max(1);

		let mut filter_by = Vec::new();

		if !session.has(EmotePermission::ViewUnlisted) {
			filter_by.push("flag_public_listed: true".to_owned());
			filter_by.push("flag_private: false".to_owned());
		}

		if let Some(tags) = tags {
			if !tags.tags.is_empty() {
				let condition = match tags.match_ {
					TagsMatch::All => " && ",
					TagsMatch::Any => " || ",
				};

				filter_by.push(format!(
					"({})",
					tags.tags.into_iter().map(|t| format!("tags:={}", t)).join(condition)
				));
			}
		}

		if let Some(filters) = filters {
			if let Some(animated) = filters.animated {
				filter_by.push(format!("flag_animated: {}", animated));
			}

			if let Some(default_zero_width) = filters.default_zero_width {
				filter_by.push(format!("flag_default_zero_width: {}", default_zero_width));
			}

			if let Some(nsfw) = filters.nsfw {
				filter_by.push(format!("flag_nsfw: {}", nsfw));
			}

			if let Some(approved_personal) = filters.approved_personal {
				filter_by.push(format!("flag_approved_personal: {}", approved_personal));
			}

			if let (Some(true), Some(query)) = (filters.exact_match, &query) {
				let sanitized = query.replace('`', "");
				let sanitized = sanitized.trim_end_matches('\\');
				if !sanitized.is_empty() {
					filter_by.push(format!("default_name: `{}`", sanitized));
				}
			}
		}

		let mut sort_by = vec!["_text_match(buckets: 10):desc".to_owned()];

		match sort.sort_by {
			SortBy::TrendingDaily => {
				sort_by.push(format!("score_trending_day:{}", sort.order));
				filter_by.push("score_trending_day:>0".to_owned());
			}
			SortBy::TrendingWeekly => {
				sort_by.push(format!("score_trending_week:{}", sort.order));
				filter_by.push("score_trending_week:>0".to_owned());
			}
			SortBy::TrendingMonthly => {
				sort_by.push(format!("score_trending_month:{}", sort.order));
				filter_by.push("score_trending_month:>0".to_owned());
			}
			SortBy::TopDaily => {
				sort_by.push(format!("score_top_daily:{}", sort.order));
				filter_by.push("score_top_daily:>0".to_owned());
			}
			SortBy::TopWeekly => {
				sort_by.push(format!("score_top_weekly:{}", sort.order));
				filter_by.push("score_top_weekly:>0".to_owned());
			}
			SortBy::TopMonthly => {
				sort_by.push(format!("score_top_monthly:{}", sort.order));
				filter_by.push("score_top_monthly:>0".to_owned());
			}
			SortBy::TopAllTime => {
				sort_by.push(format!("score_top_all_time:{}", sort.order));
				filter_by.push("score_top_all_time:>0".to_owned());
			}
			SortBy::NameAlphabetical => {
				sort_by.push(format!("default_name:{}", sort.order));
			}
			SortBy::UploadDate => {
				sort_by.push(format!("created_at:{}", sort.order));
			}
		}

		let options = SearchOptions::builder()
			.query_by(vec!["default_name".to_owned(), "tags".to_owned()])
			.query(query.unwrap_or("*".to_owned()))
			.query_by_weights(vec![4, 1])
			.per_page(per_page)
			.page(page)
			.filter_by(Some(filter_by.join(" && ")))
			.sort_by(sort_by)
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
			.load_many(result.hits.iter().copied())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emotes"))?;

		let result = SearchResult {
			items: sorted_results(result.hits, emotes)
				.into_iter()
				.map(|e| Emote::from_db(e, &global.config.api.cdn_origin))
				.collect(),
			total_count: result.found,
			page_count: result.found.div_ceil(per_page as u64).min(100),
		};

		Ok(result)
	}
}
