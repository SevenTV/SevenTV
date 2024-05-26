use axum::response::IntoResponse;

use crate::http::error::ApiError;

#[derive(utoipa::OpenApi)]
#[openapi(paths(handler))]
pub struct Docs;

#[utoipa::path(
    post,
    path = "/v3/gql",
    tag = "gql",
    responses(
        (status = 200, description = "Returns the GraphQL API response", content_type = "application/json"),
    ),
)]
pub async fn handler() -> Result<impl IntoResponse, ApiError> {
	Ok("Hello, World!")
}
