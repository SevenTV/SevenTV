use std::sync::Arc;

use async_graphql::SDLExportOptions;
use axum::Router;

use crate::global::Global;

mod gql;
mod rest;

pub fn routes(global: &Arc<Global>) -> Router<Arc<Global>> {
	Router::new().nest("/gql", gql::routes(global)).nest("/", rest::routes())
}

pub fn export_gql_schema() -> String {
	gql::schema(None).sdl_with_options(
		SDLExportOptions::default()
			.federation()
			.include_specified_by()
			.sorted_arguments()
			.sorted_enum_items()
			.sorted_fields(),
	)
}
