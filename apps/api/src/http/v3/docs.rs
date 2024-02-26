use std::sync::Arc;

use hyper::body::{Bytes, Incoming};
use scuffle_utils::http::{router::{builder::RouterBuilder, Router}, RouteError};
use shared::http::Body;
use utoipa::OpenApi;

use crate::{global::Global, http::error::ApiError};

use super::ApiDoc;

pub fn routes(_: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
    Router::builder()
        .get("/", get_docs)
}

#[utoipa::path(
    get,
    path = "/v3/docs",
    responses(
        (status = 200, description = "Returns swagger documentation", content_type = "application/json"),
    ),
)]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/docs/docs.go#L24
pub async fn get_docs(_: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    let docs = ApiDoc::openapi().to_pretty_json().unwrap();

    Ok(hyper::Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::Left(Bytes::from(docs).into()))
        .unwrap())
}
