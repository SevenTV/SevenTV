use std::sync::Arc;

use axum::response::IntoResponse;
use axum::routing::post;
use axum::Router;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

#[derive(utoipa::OpenApi)]
#[openapi(paths(create_entitlement), components(schemas(XEntitlementData)))]
pub struct Docs;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/", post(create_entitlement))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct XEntitlementData {}

#[utoipa::path(
    post,
    path = "/v3/entitlements",
    tag = "entitlements",
    request_body = XEntitlementData,
    responses(
        (status = 201, description = "Entitlement Created"),
    ),
)]
#[tracing::instrument]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/entitlements/entitlements.create.go#L34
pub async fn create_entitlement() -> Result<impl IntoResponse, ApiError> {
	// This endpoint was previously only used by admins.
	Ok(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
}
