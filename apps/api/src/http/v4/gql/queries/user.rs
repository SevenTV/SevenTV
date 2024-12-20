use std::sync::Arc;

use async_graphql::{Context, Object};
use shared::database::user::UserId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::RateLimitGuard;
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::{Platform, SearchResult, User};
use crate::search::{search, sorted_results, SearchOptions};

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
	#[tracing::instrument(skip_all, name = "UserQuery::me")]
	async fn me(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		let Some(user_id) = session.user_id() else {
			return Ok(None);
		};

		let user = global
			.user_loader
			.load(global, user_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(user.map(Into::into))
	}

	#[tracing::instrument(skip_all, name = "UserQuery::user")]
	async fn user(&self, ctx: &Context<'_>, id: UserId) -> Result<Option<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		let Some(user) = global
			.user_loader
			.load(global, id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
		else {
			return Ok(None);
		};

		Ok(session.can_view(&user).then(|| user.into()))
	}

	#[tracing::instrument(skip_all, name = "UserQuery::user_by_connection")]
	async fn user_by_connection(
		&self,
		ctx: &Context<'_>,
		platform: Platform,
		platform_id: String,
	) -> Result<Option<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		let platform = shared::database::user::connection::Platform::from(platform);

		let user = match global
			.user_by_platform_id_loader
			.load((platform, platform_id))
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
		{
			Some(u) => u,
			None => return Ok(None),
		};

		let full_user = global
			.user_loader
			.load_user(global, user)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(session.can_view(&full_user).then(|| full_user.into()))
	}

	#[graphql(guard = "RateLimitGuard::search(1)")]
	#[tracing::instrument(skip_all, name = "UserQuery::search")]
	async fn search(
		&self,
		ctx: &Context<'_>,
		#[graphql(validator(max_length = 100))] query: String,
		#[graphql(validator(maximum = 100))] page: Option<u32>,
		#[graphql(validator(minimum = 1, maximum = 100))] per_page: Option<u32>,
	) -> Result<SearchResult<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let per_page = per_page.unwrap_or(30);

		let options = SearchOptions::builder()
			.query(query)
			.query_by(vec![
				"twitch_names".to_owned(),
				"kick_names".to_owned(),
				"google_names".to_owned(),
				"discord_names".to_owned(),
			])
			.query_by_weights(vec![4, 1, 1, 1])
			.sort_by(vec![
				"_text_match(buckets: 3):desc".to_owned(),
				"role_rank:desc".to_owned(),
				"_text_match(buckets: 10):desc".to_owned(),
			])
			.page(page)
			.per_page(per_page)
			.prioritize_exact_match(true)
			.exaustive(true)
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

		Ok(SearchResult {
			items: sorted_results(result.hits, users).into_iter().map(Into::into).collect(),
			total_count: result.found,
			page_count: result.found.div_ceil(per_page as u64).min(100),
		})
	}
}
