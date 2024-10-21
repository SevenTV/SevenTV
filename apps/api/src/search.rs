use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use itertools::Itertools;
use shared::typesense::types::TypesenseCollection;
use typed_builder::TypedBuilder;
use typesense_rs::apis::documents_api::{SearchCollectionError, SearchCollectionParams};
use typesense_rs::apis::Api;

use crate::global::Global;

#[derive(Debug)]
pub enum SearchError {
	Search(typesense_rs::apis::Error<SearchCollectionError>),
}

impl From<typesense_rs::apis::Error<SearchCollectionError>> for SearchError {
	fn from(value: typesense_rs::apis::Error<SearchCollectionError>) -> Self {
		Self::Search(value)
	}
}

impl Display for SearchError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Search(typesense_rs::apis::Error::ResponseError(e)) => {
				write!(f, "status code {}, content: {}", e.status, e.content)
			}
			Self::Search(e) => write!(f, "{e}"),
		}
	}
}

impl std::error::Error for SearchError {}

#[derive(TypedBuilder, Debug, Clone)]
#[builder(field_defaults(setter(into)))]
pub struct SearchOptions {
	pub query: String,
	#[builder(default)]
	pub query_by: Vec<String>,
	#[builder(default)]
	pub prefix: Option<Vec<String>>,
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
	pub prioritize_num_matching_fields: Option<bool>,
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
#[allow(dead_code)]
pub struct SearchResult<V> {
	pub hits: Vec<V>,
	pub found: u64,
	pub search_time_ms: u64,
}

#[tracing::instrument(skip_all, fields(collection_name = T::COLLECTION_NAME))]
pub async fn search<T: TypesenseCollection>(
	global: &Arc<Global>,
	options: SearchOptions,
) -> Result<SearchResult<T::Id>, SearchError> {
	let resp = global
		.typesense
		.documents_api()
		.search_collection(
			SearchCollectionParams::builder()
				.collection_name(T::COLLECTION_NAME.to_owned())
				.q(options.query)
				.query_by(options.query_by.join(","))
				.maybe_prefix(options.prefix.map(|p| p.join(",")))
				.maybe_query_by_weights(options.query_by_weights.map(|w| w.iter().map(|i| i.to_string()).join(",")))
				.maybe_filter_by(options.filter_by)
				.maybe_sort_by(options.sort_by.map(|s| s.join(",")))
				.maybe_prioritize_exact_match(options.prioritize_exact_match)
				.maybe_prioritize_token_position(options.prioritize_token_position)
				.maybe_prioritize_num_matching_fields(options.prioritize_num_matching_fields)
				.maybe_page(options.page.map(|p| i32::try_from(p).unwrap_or(i32::MAX)))
				.maybe_per_page(options.per_page.map(|p| i32::try_from(p).unwrap_or(i32::MAX)))
				.maybe_exhaustive_search(options.exaustive)
				.maybe_num_typos(options.typo_limit.map(|t| t.iter().map(|i| i.to_string()).join(",")))
				.include_fields("id".to_string())
				.highlight_fields("false".to_string())
				.build(),
		)
		.await?;

	Ok(SearchResult {
		hits: resp
			.hits
			.into_iter()
			.flatten()
			.filter_map(|h| serde_json::from_value(h.document?.remove("id")?).ok())
			.collect(),
		found: resp.found.unwrap_or(0).max(0) as u64,
		search_time_ms: resp.search_time_ms.unwrap_or(0).max(0) as u64,
	})
}

pub fn sorted_results<'a, K: std::hash::Hash + Eq + 'a, V: 'a, B: Borrow<K> + 'a, H: IntoIterator<Item = B>>(
	hits: H,
	mut loaded: HashMap<K, V>,
) -> impl IntoIterator<Item = V> + 'a
where
	H::IntoIter: 'a,
{
	hits.into_iter().filter_map(move |h| loaded.remove(h.borrow()))
}
