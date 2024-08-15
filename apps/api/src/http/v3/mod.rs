use std::sync::Arc;
use std::time::Duration;

use axum::http::HeaderName;
use axum::Router;
use hyper::Method;
use tower_http::cors::{AllowCredentials, AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders, MaxAge};
use utoipa::OpenApi;

use crate::global::Global;

pub mod docs;
pub mod emote_set_loader;
pub mod gql;
pub mod rest;

pub fn docs() -> utoipa::openapi::OpenApi {
	#[derive(OpenApi)]
	#[openapi(
        info(
            title = "7TV",
            version = "3.0.0",
            contact(
                email = "support@7tv.app",
            ),
            license(
                name = "Apache 2.0 with Commons Clause",
                url = "https://github.com/SevenTV/SevenTV/blob/main/licenses/CC_APACHE2_LICENSE",
            ),
            description = include_str!("DESCRIPTION.md"),
        ),
        servers(
            (url = "https://7tv.io", description = "Production"),
        ),
    )]
	struct Docs;

	let mut docs = Docs::openapi();
	docs.merge(docs::Docs::openapi());
	docs.merge(gql::Docs::openapi());
	docs.merge(rest::types::Docs::openapi());
	docs.merge(rest::config::Docs::openapi());
	docs.merge(rest::auth::Docs::openapi());
	docs.merge(rest::emotes::Docs::openapi());
	docs.merge(rest::emote_sets::Docs::openapi());
	docs.merge(rest::users::Docs::openapi());
	docs.merge(rest::entitlements::Docs::openapi());
	docs
}

const ALLOWED_CORS_HEADERS: [&'static str; 8] = [
	"content-type",
	"content-length",
	"accept-encoding",
	"authorization",
	"cookie",
	"x-emote-data",
	"x-seventv-platform",
	"x-seventv-version",
];

fn cors_layer(global: &Arc<Global>) -> CorsLayer {
	let website_origin = global.config.api.website_origin.clone();
	let api_origin = global.config.api.api_origin.clone();
	let allow_credentials = AllowCredentials::predicate(move |origin, _| {
		origin
			.to_str()
			.map(|o| o == website_origin || o == api_origin)
			.unwrap_or_default()
	});

	CorsLayer::new()
		.allow_origin(AllowOrigin::mirror_request())
		.allow_credentials(allow_credentials)
		.allow_methods(AllowMethods::list([
			Method::GET,
			Method::POST,
			Method::PUT,
			Method::PATCH,
			Method::DELETE,
		]))
		.allow_headers(AllowHeaders::list(
			ALLOWED_CORS_HEADERS.into_iter().map(HeaderName::from_static),
		))
		.expose_headers(ExposeHeaders::list([HeaderName::from_static("x-access-token")]))
		.max_age(MaxAge::exact(Duration::from_secs(7200)))
}

pub fn routes(global: &Arc<Global>) -> Router<Arc<Global>> {
	Router::new()
		.nest("/docs", docs::routes())
		.nest("/", rest::routes())
		.nest("/gql", gql::routes(global))
		.layer(cors_layer(global))
}
