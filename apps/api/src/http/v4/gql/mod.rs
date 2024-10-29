use std::sync::Arc;

use async_graphql::{extensions, EmptyMutation, EmptySubscription, Schema};
use axum::response::{self, IntoResponse};
use axum::{
	routing::{get, post},
	Extension, Router,
};

use crate::http::guards::RateLimitResponseStore;
use crate::{global::Global, http::middleware::session::Session};

mod queries;
mod types;

pub type V3Schema = Schema<queries::Query, EmptyMutation, EmptySubscription>;

pub fn schema(global: Option<Arc<Global>>) -> V3Schema {
	let mut schema = Schema::build(queries::Query::default(), EmptyMutation, EmptySubscription)
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
	Extension(schema): Extension<V3Schema>,
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
