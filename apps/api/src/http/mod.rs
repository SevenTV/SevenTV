use std::sync::Arc;

use hyper::body::Incoming;
use hyper::StatusCode;
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::{error_handler, RouteError};
use shared::http::{cors_middleware, Body};

use self::error::ApiError;
use crate::global::Global;

mod error;
pub mod v3;

fn routes(global: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
	let weak = Arc::downgrade(global);

	Router::builder()
		.data(weak)
		.error_handler(|req, err| async move { error_handler(req, err).await.map(Body::Left) })
		.middleware(cors_middleware(&global.config().api.cors))
		// Handle the v3 API, we have to use a wildcard because of the path format.
		.scope("/v3", v3::routes(global))
		// Not found handler.
		.not_found(|_| async move { Err((StatusCode::NOT_FOUND, "not found").into()) })
}

#[tracing::instrument(name = "api", level = "info", skip(global))]
pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let config = global.config();

	shared::http::run(global.ctx(), &config.api.http, routes(&global).build(), |_| true).await?;

	Ok(())
}
