use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use itertools::Itertools;
use shared::typesense::types::TypesenseCollection;
use typed_builder::TypedBuilder;
use typesense_rs::apis::documents_api::{
	MultiSearchError, MultiSearchParams, SearchCollectionError, SearchCollectionParams,
};
use typesense_rs::apis::Api;
use typesense_rs::models::{MultiSearchCollectionParameters, MultiSearchSearchesParameter};

use crate::global::Global;

#[derive(Debug)]
pub enum SearchError {
	Search(typesense_rs::apis::Error<SearchCollectionError>),
	MultiSearch(typesense_rs::apis::Error<MultiSearchError>),
}

impl From<typesense_rs::apis::Error<SearchCollectionError>> for SearchError {
	fn from(value: typesense_rs::apis::Error<SearchCollectionError>) -> Self {
		Self::Search(value)
	}
}

impl From<typesense_rs::apis::Error<MultiSearchError>> for SearchError {
	fn from(value: typesense_rs::apis::Error<MultiSearchError>) -> Self {
		Self::MultiSearch(value)
	}
}

impl Display for SearchError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Search(typesense_rs::apis::Error::ResponseError(e)) => {
				write!(f, "status code {}, content: {}", e.status, e.content)
			}
			Self::MultiSearch(typesense_rs::apis::Error::ResponseError(e)) => {
				write!(f, "status code {}, content: {}", e.status, e.content)
			}
			Self::Search(e) => write!(f, "{e}"),
			Self::MultiSearch(e) => write!(f, "{e}"),
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

/// This is rather ugly code but there is no way to allow an arbitrary number of
/// generics without a macro
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all, fields(collection1_name = C1::COLLECTION_NAME, collection2_name = C2::COLLECTION_NAME))]
pub async fn multi_search_2<C1: TypesenseCollection, C2: TypesenseCollection>(
	global: &Arc<Global>,
	options1: SearchOptions,
	options2: SearchOptions,
) -> Result<(SearchResult<C1::Id>, SearchResult<C2::Id>), SearchError> {
	let searches = [
		(C1::COLLECTION_NAME.to_owned(), options1),
		(C2::COLLECTION_NAME.to_owned(), options2),
	]
	.into_iter()
	.map(|(collection, options)| {
		let mut params = MultiSearchCollectionParameters::new(collection);
		params.q = Some(options.query);
		params.query_by = Some(options.query_by.join(","));
		params.prefix = options.prefix.map(|p| p.join(","));
		params.query_by_weights = options.query_by_weights.map(|w| w.iter().map(|i| i.to_string()).join(","));
		params.filter_by = options.filter_by;
		params.sort_by = options.sort_by.map(|s| s.join(","));
		params.prioritize_exact_match = options.prioritize_exact_match;
		params.prioritize_token_position = options.prioritize_token_position;
		params.prioritize_num_matching_fields = options.prioritize_num_matching_fields;
		params.page = options.page.map(|p| i32::try_from(p).unwrap_or(i32::MAX));
		params.per_page = options.per_page.map(|p| i32::try_from(p).unwrap_or(i32::MAX));
		params.exhaustive_search = options.exaustive;
		params.num_typos = options.typo_limit.map(|t| t.iter().map(|i| i.to_string()).join(","));
		params.include_fields = Some("id".to_string());
		params.highlight_fields = Some("false".to_string());
		params
	})
	.collect();

	let mut resp = global
		.typesense
		.documents_api()
		.multi_search(
			MultiSearchParams::builder()
				.multi_search_searches_parameter(MultiSearchSearchesParameter { searches })
				.build(),
		)
		.await?;

	let pos = resp.results.iter().position(|r| {
		r.request_params
			.as_ref()
			.is_some_and(|p| p.collection_name == C1::COLLECTION_NAME)
	});
	let result1 = pos.map(|i| resp.results.remove(i)).unwrap_or_default();

	let result1 = SearchResult {
		hits: result1
			.hits
			.into_iter()
			.flatten()
			.filter_map(|h| serde_json::from_value(h.document?.remove("id")?).ok())
			.collect(),
		found: result1.found.unwrap_or(0).max(0) as u64,
		search_time_ms: result1.search_time_ms.unwrap_or(0).max(0) as u64,
	};

	let pos = resp.results.iter().position(|r| {
		r.request_params
			.as_ref()
			.is_some_and(|p| p.collection_name == C2::COLLECTION_NAME)
	});
	let result2 = pos.map(|i| resp.results.remove(i)).unwrap_or_default();

	let result2 = SearchResult {
		hits: result2
			.hits
			.into_iter()
			.flatten()
			.filter_map(|h| serde_json::from_value(h.document?.remove("id")?).ok())
			.collect(),
		found: result2.found.unwrap_or(0).max(0) as u64,
		search_time_ms: result2.search_time_ms.unwrap_or(0).max(0) as u64,
	};

	Ok((result1, result2))
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
