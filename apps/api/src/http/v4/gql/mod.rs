use std::sync::Arc;

use async_graphql::{extensions, EmptySubscription, Schema};
use axum::response::{self, IntoResponse};
use axum::routing::{get, post};
use axum::{Extension, Router};

use crate::global::Global;
use crate::http::guards::RateLimitResponseStore;
use crate::http::middleware::session::Session;

mod mutations;
mod queries;
mod types;

pub type V4Schema = Schema<queries::Query, mutations::Mutation, EmptySubscription>;

pub fn schema(global: Option<Arc<Global>>) -> V4Schema {
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

pub fn routes(global: &Arc<Global>) -> Router<Arc<Global>> {
	Router::new()
		.route("/", post(graphql_handler))
		.route("/playground", get(playground))
		.layer(Extension(schema(Some(Arc::clone(global)))))
}

pub async fn graphql_handler(
	Extension(schema): Extension<V4Schema>,
	Extension(session): Extension<Session>,
	req: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
	let req = req.into_inner().data(session).data(RateLimitResponseStore::new());

	schema.execute(req).await.into()
}

pub async fn playground() -> impl IntoResponse {
	response::Html(
		async_graphql::http::GraphiQLSource::build()
			.endpoint("/v4/gql")
			.title("7TV API v4 GraphQL Playground")
			.finish(),
	)
}
