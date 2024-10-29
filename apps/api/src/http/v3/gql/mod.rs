use std::sync::Arc;

use async_graphql::{extensions, BatchRequest, BatchResponse, EmptySubscription, Schema};
use async_graphql_axum::rejection::GraphQLRejection;
use axum::extract::FromRequest;
use axum::response::{self, IntoResponse};
use axum::routing::{any, get};
use axum::{Extension, Router};

use crate::global::Global;
use crate::http::guards::RateLimitResponseStore;
use crate::http::middleware::session::Session;

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

#[axum::debug_handler]
#[utoipa::path(post, path = "/v3/gql", tag = "gql")]
#[tracing::instrument(skip_all, name = "v3_gql", fields(batch_size))]
pub async fn graphql_handler(
	Extension(schema): Extension<V3Schema>,
	Extension(session): Extension<Session>,
	request: axum::extract::Request,
) -> Result<axum::response::Response, ApiError> {
	if request.method() == hyper::Method::GET {
		// There is no reason to use GET requests in a web-browser, and it's likely a
		// CSRF attempt. We do not allow this because someone can redirect a user to
		// this endpoint, and they won't know or could embed it as an iframe or image
		// tag or something. The user would not know and it would execute graphql
		// queries on their behalf. Therefore we deny any GET request with a cookie,
		// origin, or referrer header.
		if request.headers().get(hyper::header::COOKIE).is_some()
			|| request.headers().get(hyper::header::ORIGIN).is_some()
			|| request.headers().get(hyper::header::REFERER).is_some()
		{
			return Err(ApiError::bad_request(
				ApiErrorCode::BadRequest,
				"You cannot use GET requests in a web-browser, use POST instead.",
			));
		}
	}

	let req = match async_graphql_axum::GraphQLBatchRequest::<GraphQLRejection>::from_request(request, &()).await {
		Ok(req) => req,
		Err(err) => return Ok(err.into_response()),
	};

	let batch_size = req.0.iter().count();
	tracing::Span::current().record("batch_size", batch_size);

	if let BatchRequest::Single(req) = &req.0 {
		if req.query == "forsen" {
			return Ok(
				async_graphql_axum::GraphQLResponse(BatchResponse::Single(async_graphql::Response::new(
					async_graphql::Value::String("forsen1".to_string()),
				)))
				.into_response(),
			);
		}
	}

	if batch_size > 30 {
		return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "batch size too large"));
	}

	let req = req.into_inner().data(session).data(RateLimitResponseStore::new());

	let response = schema.execute_batch(req).await;

	Ok(async_graphql_axum::GraphQLResponse::from(response).into_response())
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
