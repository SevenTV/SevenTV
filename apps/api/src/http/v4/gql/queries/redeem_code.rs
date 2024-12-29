use std::sync::Arc;

use async_graphql::Context;
use shared::database::product::codes::RedeemCodeId;
use shared::database::role::permissions::{RateLimitResource, UserPermission};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::{PermissionGuard, RateLimitGuard};
use crate::http::v4::gql::types::{RedeemCode, SearchResult};
use crate::search::{search, sorted_results, SearchOptions};

#[derive(Default)]
pub struct RedeemCodeQuery;

#[async_graphql::Object]
impl RedeemCodeQuery {
	#[tracing::instrument(skip_all, name = "RedeemCodeQuery::redeem_code")]
	#[graphql(guard = "PermissionGuard::one(UserPermission::ManageBilling)")]
	async fn redeem_code(&self, ctx: &Context<'_>, id: RedeemCodeId) -> Result<Option<RedeemCode>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let code = global
			.redeem_code_by_id_loader
			.load(id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load redeem code"))?;

		Ok(code.map(Into::into))
	}

	#[tracing::instrument(skip_all, name = "RedeemCodeQuery::redeem_codes")]
	#[graphql(
		guard = "PermissionGuard::one(UserPermission::ManageBilling).and(RateLimitGuard::new(RateLimitResource::Search, 1))"
	)]
	async fn redeem_codes(
		&self,
		ctx: &Context<'_>,
		#[graphql(validator(max_length = 100))] query: Option<String>,
		remaining_uses: Option<bool>,
		#[graphql(validator(maximum = 100))] page: Option<u32>,
		#[graphql(validator(minimum = 1, maximum = 250))] per_page: Option<u32>,
	) -> Result<SearchResult<RedeemCode>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let per_page = per_page.unwrap_or(30);
		let page = page.unwrap_or_default().max(1);

		let mut filters = vec![];

		if let Some(true) = remaining_uses {
			filters.push("remaining_uses:>0".to_owned());
		}

		let options = SearchOptions::builder()
			.query_by(vec!["code".to_owned(), "name".to_owned(), "tags".to_owned()])
			.query(query.unwrap_or("*".to_owned()))
			.query_by_weights(vec![1, 1, 1])
			.filter_by(filters.join(" && "))
			.per_page(per_page)
			.page(page)
			.sort_by(vec!["_text_match(buckets: 10):desc".to_owned(), "created_at:desc".to_owned()])
			.exaustive(true)
			.build();

		let result = search::<shared::typesense::types::product::codes::RedeemCode>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to search")
			})?;

		let codes = global
			.redeem_code_by_id_loader
			.load_many(result.hits.iter().copied())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load redeem codes"))?;

		let result = SearchResult {
			items: sorted_results(result.hits, codes).into_iter().map(Into::into).collect(),
			total_count: result.found,
			page_count: result.found.div_ceil(per_page as u64).min(100),
		};

		Ok(result)
	}
}
