use std::sync::{Arc, Weak};

use hyper::body::Incoming;
use hyper::StatusCode;
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::{error_handler, RouteError};
use shared::http::{cors_middleware, Body};

use self::error::ApiError;
use crate::global::Global;

mod cookies;
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

pub trait RequestGlobalExt<E> {
	fn get_global<G: Sync + Send + 'static, B: From<tokio_util::bytes::Bytes>>(
		&self,
	) -> std::result::Result<Arc<G>, RouteError<E, B>>;
}

impl<E, B> RequestGlobalExt<E> for hyper::Request<B> {
	fn get_global<G: Sync + Send + 'static, B2: From<tokio_util::bytes::Bytes>>(
		&self,
	) -> std::result::Result<Arc<G>, RouteError<E, B2>> {
		Ok(self
			.extensions()
			.get::<Weak<G>>()
			.expect("global state not set")
			.upgrade()
			.ok_or((StatusCode::INTERNAL_SERVER_ERROR, "failed to upgrade global state"))?)
	}
}

pub trait RequestQueryParamExt {
	fn query_param(&self, key: &str) -> Option<String> {
		self.query_params().find(|(k, _)| *k == key).map(|(_, v)| v)
	}

	fn query_params(&self) -> impl Iterator<Item = (&str, String)>;
}

impl<B> RequestQueryParamExt for hyper::Request<B> {
	fn query_params(&self) -> impl Iterator<Item = (&str, String)> {
		self.uri().query().unwrap_or("").split('&').filter_map(|param| {
			let mut parts = param.splitn(2, '=');
			let key = parts.next()?;
			let value = parts.next().unwrap_or("");
			Some((
				key,
				urlencoding::decode(value)
					.map(|s| s.into_owned())
					.unwrap_or_else(|_| value.to_string()),
			))
		})
	}
}
