use std::sync::Arc;

use async_graphql::{extensions, EmptySubscription, Schema};
use axum::{
	response::{self, IntoResponse}, routing::{get, post}, Extension, Router
};

use crate::{global::Global, http::middleware::auth::AuthSession};

mod guards;
mod mutations;
mod queries;

pub fn routes(global: &Arc<Global>) -> Router<Arc<Global>> {
	Router::new()
		.route("/", post(graphql_handler))
		.route("/playground", get(playground))
		.layer(Extension(schema(Arc::clone(global))))
}

pub type V3Schema = Schema<queries::Query, mutations::Mutation, EmptySubscription>;

pub fn schema(global: Arc<Global>) -> V3Schema {
	Schema::build(queries::Query::default(), mutations::Mutation::default(), EmptySubscription)
		.data(global)
		.enable_federation()
		.enable_subscription_in_federation()
		.extension(extensions::Analyzer)
		.extension(extensions::Tracing)
		.limit_complexity(400) // We don't want to allow too complex queries to be executed
		.finish()
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(graphql_handler, playground))]
pub struct Docs;

#[utoipa::path(post, path = "/v3/gql", tag = "gql")]
pub async fn graphql_handler(
	schema: Extension<V3Schema>,
	auth: Option<Extension<AuthSession>>,
	req: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
	let mut req = req.into_inner();
	if let Some(Extension(session)) = auth {
		req = req.data(session);
	}
	schema.execute(req).await.into()
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
