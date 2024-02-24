use std::sync::Arc;

use hyper::body::Incoming;
use hyper::StatusCode;
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::{error_handler, RouteError};
use shared::http::{cors_middleware, Body};

use self::error::EventError;
use crate::global::Global;

mod error;
pub mod socket;
pub mod v3;

fn routes(global: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<EventError>> {
	let weak = Arc::downgrade(global);

	let mut builder = Router::builder()
		.data(weak)
		.error_handler(|req, err| async move { error_handler(req, err).await.map(Body::Left) });

	if let Some(cors) = &global.config().api.cors {
		builder = builder.middleware(cors_middleware(cors));
	}

	builder
		// Handle the v3 API, we have to use a wildcard because of the path format.
		.any("/v3*", v3::handle)
		// Not found handler.
		.not_found(|_| async move { Err((StatusCode::NOT_FOUND, "not found").into()) })
}

#[tracing::instrument(name = "api", level = "info", skip(global))]
pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let config = global.config();

	let accept_connection_fn = |_| {
		if let Some(limit) = global.config().api.connection_limit {
			let active = global.active_connections();
			if active >= limit {
				tracing::debug!("connection limit reached");
				return false;
			}
		}

		true
	};

	shared::http::run(global.ctx(), &config.api.http, routes(&global).build(), accept_connection_fn).await?;

	Ok(())
}
