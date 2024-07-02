use std::sync::Arc;

use itertools::Itertools;
use shared::typesense::types::TypesenseCollection;
use typed_builder::TypedBuilder;
use typesense_codegen::apis::documents_api::SearchCollectionError;
use typesense_codegen::models::SearchParameters;

use crate::global::Global;

#[derive(thiserror::Error, Debug)]
pub enum SearchError {
	#[error("failed to search: {0}")]
	Search(#[from] typesense_codegen::apis::Error<SearchCollectionError>),
}

#[derive(TypedBuilder, Debug, Clone)]
#[builder(field_defaults(setter(into)))]
pub struct SearchOptions {
	pub query: String,
	pub query_by: Vec<String>,
	#[builder(default)]
	pub query_by_weights: Option<Vec<i32>>,
	#[builder(default)]
	pub filter_by: Option<String>,
	#[builder(default)]
	pub sort_by: Option<Vec<String>>,
	#[builder(default)]
	pub prioritize_exact_match: Option<bool>,
	#[builder(default)]
	pub prioritize_token_position: Option<bool>,
	#[builder(default)]
	pub page: Option<u32>,
	#[builder(default)]
	pub per_page: Option<u32>,
	#[builder(default)]
	pub exaustive: Option<bool>,
	#[builder(default)]
	pub typo_limit: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct SearchResult<T: TypesenseCollection> {
	pub hits: Vec<T::Id>,
	pub found: u64,
	pub search_time_ms: u64,
}

pub async fn search<T: TypesenseCollection>(
	global: &Arc<Global>,
	options: SearchOptions,
) -> Result<SearchResult<T>, SearchError> {
	#[derive(serde::Deserialize)]
	struct SearchHit<T: TypesenseCollection> {
		id: T::Id,
	}

	let resp = typesense_codegen::apis::documents_api::search_collection::<SearchHit<T>>(
		global.typesense(),
		T::COLLECTION_NAME,
		SearchParameters {
			q: options.query,
			query_by: options.query_by.join(","),
			query_by_weights: options.query_by_weights.map(|w| w.iter().map(|i| i.to_string()).join(",")),
			filter_by: options.filter_by,
			sort_by: options.sort_by.map(|s| s.join(",")),
			prioritize_exact_match: options.prioritize_exact_match,
			prioritize_token_position: options.prioritize_token_position,
			page: options.page.map(|p| i32::try_from(p).unwrap_or(i32::MAX)),
			per_page: options.per_page.map(|p| i32::try_from(p).unwrap_or(i32::MAX)),
			exhaustive_search: options.exaustive,
			num_typos: options.typo_limit.map(|t| t.iter().map(|i| i.to_string()).join(",")),
			include_fields: Some("id".to_string()),
			highlight_fields: Some("false".to_string()),
			..Default::default()
		},
	)
	.await?;

	Ok(SearchResult {
		hits: resp.hits.into_iter().flatten().filter_map(|h| Some(h.document?.id)).collect(),
		found: resp.found.unwrap_or(0).min(0) as u64,
		search_time_ms: resp.search_time_ms.unwrap_or(0).min(0) as u64,
	})
}
