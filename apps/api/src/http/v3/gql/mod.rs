use std::sync::Arc;

use async_graphql::{extensions, BatchRequest, BatchResponse, EmptySubscription, Schema};
use axum::response::{self, IntoResponse};
use axum::routing::{any, get};
use axum::{Extension, Router};
use guards::RateLimitResponseStore;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;

mod guards;
mod metrics;
mod mutations;
mod queries;
mod types;

pub fn routes(global: &Arc<Global>) -> Router<Arc<Global>> {
	Router::new()
		.route("/", any(graphql_handler))
		.route("/playground", get(playground))
		.layer(Extension(schema(Some(Arc::clone(global)))))
}

pub type V3Schema = Schema<queries::Query, mutations::Mutation, EmptySubscription>;

pub fn schema(global: Option<Arc<Global>>) -> V3Schema {
	let mut schema = Schema::build(queries::Query::default(), mutations::Mutation::default(), EmptySubscription)
		.enable_federation()
		.enable_subscription_in_federation()
		.extension(extensions::Analyzer)
		.extension(extensions::ApolloTracing)
		.extension(metrics::ErrorMetrics)
		.limit_complexity(400); // We don't want to allow too complex queries to be executed

	if let Some(global) = global {
		schema = schema.data(global);
	}

	schema.finish()
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(graphql_handler, playground))]
pub struct Docs;

#[utoipa::path(post, path = "/v3/gql", tag = "gql")]
#[tracing::instrument(skip_all, name = "v3_gql", fields(batch_size))]
pub async fn graphql_handler(
	Extension(schema): Extension<V3Schema>,
	Extension(session): Extension<Session>,
	req: async_graphql_axum::GraphQLBatchRequest,
) -> Result<async_graphql_axum::GraphQLResponse, ApiError> {
	let batch_size = req.0.iter().count();
	tracing::Span::current().record("batch_size", batch_size);

	if let BatchRequest::Single(req) = &req.0 {
		if req.query == "forsen" {
			return Ok(async_graphql_axum::GraphQLResponse(BatchResponse::Single(
				async_graphql::Response::new(async_graphql::Value::String("forsen1".to_string())),
			)));
		}
	}

	if batch_size > 30 {
		return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "batch size too large"));
	}

	let req = req.into_inner().data(session).data(RateLimitResponseStore::new());

	let response = schema.execute_batch(req).await;

	Ok(response.into())
}

#[utoipa::path(get, path = "/v3/gql/playground", tag = "gql")]
pub async fn playground() -> impl IntoResponse {
	response::Html(
		async_graphql::http::GraphiQLSource::build()
			.endpoint("/v3/gql")
			.title("7TV API v3 GraphQL Playground")
			.finish(),
	)
}
