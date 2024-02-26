use hyper::body::{Bytes, Incoming};
use scuffle_utils::http::RouteError;
use shared::http::Body;
use utoipa::OpenApi;

use crate::http::error::ApiError;

use super::ApiDoc;

#[utoipa::path(
    get,
    path = "/v3/docs",
    responses(
        (status = 200, description = "Returns swagger documentation", content_type = "application/json"),
    ),
)]
pub async fn handler(_: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    let docs = ApiDoc::openapi().to_pretty_json().unwrap();

    Ok(hyper::Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::Left(Bytes::from(docs).into()))
        .unwrap())
}
