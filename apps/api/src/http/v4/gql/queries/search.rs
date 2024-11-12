use std::sync::Arc;

use async_graphql::{Context, Object};
use shared::database::role::permissions::{EmotePermission, PermissionsExt};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::RateLimitGuard;
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::{Emote, SearchResult, User};
use crate::search::{multi_search_2, sorted_results, SearchOptions};

#[derive(Default)]
pub struct SearchQuery;

#[derive(async_graphql::SimpleObject)]
pub struct SearchResultAll {
	emotes: SearchResult<Emote>,
	users: SearchResult<User>,
}

#[Object]
impl SearchQuery {
	#[graphql(guard = "RateLimitGuard::search(1)")]
	async fn all<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		query: Option<String>,
		#[graphql(validator(maximum = 100))] page: Option<u32>,
		#[graphql(validator(minimum = 1, maximum = 100))] per_page: Option<u32>,
	) -> Result<SearchResultAll, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		let per_page = per_page.unwrap_or(10);
		let page = page.unwrap_or_default().max(1);

		let mut emotes_filter_by = Vec::new();

		if !session.has(EmotePermission::ViewUnlisted) {
			emotes_filter_by.push("flag_public_listed: true".to_owned());
			emotes_filter_by.push("flag_private: false".to_owned());
		}

		let query = query.unwrap_or("*".to_owned());

		let emotes_options = SearchOptions::builder()
			.query_by(vec!["default_name".to_owned(), "tags".to_owned()])
			.query(query.clone())
			.query_by_weights(vec![4, 1])
			.per_page(per_page)
			.page(page)
			.filter_by(Some(emotes_filter_by.join(" && ")))
			.sort_by(vec![
				"_text_match(buckets: 10):desc".to_owned(),
				"score_top_all_time:desc".to_owned(),
			])
			.exaustive(true)
			.build();

		let users_options = SearchOptions::builder()
			.query_by(vec![
				"twitch_names".to_owned(),
				"kick_names".to_owned(),
				"google_names".to_owned(),
				"discord_names".to_owned(),
			])
			.query(query)
			.query_by_weights(vec![4, 1, 1, 1])
			.per_page(per_page)
			.page(page)
			.sort_by(vec!["_text_match(buckets: 10):desc".to_owned(), "role_rank:desc".to_owned()])
			.prioritize_exact_match(true)
			.exaustive(true)
			.build();

		let (emote_result, user_results) = multi_search_2::<
			shared::typesense::types::emote::Emote,
			shared::typesense::types::user::User,
		>(global, emotes_options, users_options)
		.await
		.map_err(|err| {
			tracing::error!(error = %err, "failed to search");
			ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to search")
		})?;

		let emotes = global
			.emote_by_id_loader
			.load_many(emote_result.hits.iter().copied())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emotes"))?;

		let emotes = SearchResult {
			items: sorted_results(emote_result.hits, emotes)
				.into_iter()
				.map(|e| Emote::from_db(e, &global.config.api.cdn_origin))
				.collect(),
			total_count: emote_result.found,
			page_count: emote_result.found.div_ceil(per_page as u64).min(100),
		};

		let users = global
			.user_loader
			.load_fast_many(global, user_results.hits.iter().copied())
			.await
			.map_err(|()| {
				tracing::error!("failed to load users");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load users")
			})?;

		let users = SearchResult {
			items: sorted_results(user_results.hits, users).into_iter().map(Into::into).collect(),
			total_count: user_results.found,
			page_count: user_results.found.div_ceil(per_page as u64).min(100),
		};

		Ok(SearchResultAll { emotes, users })
	}
}
