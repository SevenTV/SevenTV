use std::sync::Arc;

use async_graphql::{extensions, EmptySubscription, Schema};
use axum::response::{self, IntoResponse};
use axum::routing::{get, post};
use axum::{Extension, Router};
use guards::RateLimitResponseStore;

use crate::global::Global;
use crate::http::middleware::session::Session;

mod guards;
mod mutations;
mod queries;
mod types;

pub fn routes(global: &Arc<Global>) -> Router<Arc<Global>> {
	Router::new()
		.route("/", post(graphql_handler))
		.route("/batch", post(graphql_batch_handler))
		.route("/playground", get(playground))
		.layer(Extension(schema(Some(Arc::clone(global)))))
}

pub type V3Schema = Schema<queries::Query, mutations::Mutation, EmptySubscription>;

pub fn schema(global: Option<Arc<Global>>) -> V3Schema {
	let mut schema = Schema::build(queries::Query::default(), mutations::Mutation::default(), EmptySubscription)
		.enable_federation()
		.enable_subscription_in_federation()
		.extension(extensions::Analyzer)
		.extension(extensions::Tracing)
		.limit_complexity(400); // We don't want to allow too complex queries to be executed

	if let Some(global) = global {
		schema = schema.data(global);
	}

	schema.finish()
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(graphql_handler, graphql_batch_handler, playground))]
pub struct Docs;

#[utoipa::path(post, path = "/v3/gql", tag = "gql")]
pub async fn graphql_handler(
	Extension(schema): Extension<V3Schema>,
	Extension(session): Extension<Session>,
	req: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
	let req = req.into_inner().data(session).data(RateLimitResponseStore::new());

	schema.execute(req).await.into()
}

#[utoipa::path(post, path = "/v3/gql/batch", tag = "gql")]
pub async fn graphql_batch_handler(
	Extension(schema): Extension<V3Schema>,
	Extension(session): Extension<Session>,
	req: async_graphql_axum::GraphQLBatchRequest,
) -> async_graphql_axum::GraphQLResponse {
	let req = req.into_inner().data(session).data(RateLimitResponseStore::new());

	schema.execute_batch(req).await.into()
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
