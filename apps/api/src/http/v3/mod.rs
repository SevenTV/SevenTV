use std::sync::Arc;

use axum::{routing::{get, post_service}, Router};

use crate::global::Global;

pub mod emote_set_loader;
pub mod gql;
pub mod rest;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.nest("/", rest::routes())
		.route("/gql", post_service(async_graphql_axum::GraphQL::new(gql::schema())))
		.route("/gql/playground", get(gql::playground))
}
