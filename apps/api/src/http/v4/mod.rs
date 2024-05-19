use std::sync::Arc;

use axum::Router;
use utoipa::OpenApi;

use crate::global::Global;

mod docs;

pub fn docs() -> utoipa::openapi::OpenApi {
	#[derive(OpenApi)]
	#[openapi(
        info(
            title = "7TV",
            version = "4.0.0",
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
            (url = "https://7tv.io/v4", description = "Production"),
        ),
    )]
	struct Docs;

	let mut docs = Docs::openapi();
	docs.merge(docs::Docs::openapi());
	docs
}

pub fn routes() -> Router<Arc<Global>> {
	Router::new().nest("/docs", docs::routes())
}
