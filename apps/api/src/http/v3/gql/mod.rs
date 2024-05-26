use async_graphql::{extensions, EmptySubscription, Schema};
use axum::response::{self, IntoResponse};

mod mutations;
mod queries;

pub type V3Schema = Schema<queries::Query, mutations::Mutation, EmptySubscription>;

pub fn schema() -> V3Schema {
	Schema::build(queries::Query::default(), mutations::Mutation::default(), EmptySubscription)
        .enable_federation()
        .enable_subscription_in_federation()
        .extension(extensions::Analyzer)
        .extension(extensions::Tracing)
        .limit_complexity(400) // We don't want to allow too complex queries to be executed
        .finish()
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
