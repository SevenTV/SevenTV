use axum::async_trait;
use axum::extract::rejection::{PathRejection, QueryRejection};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use hyper::StatusCode;
use serde::de::DeserializeOwned;

use super::error::ApiError;
use crate::http::error::ApiErrorCode;

pub struct Path<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for Path<T>
where
	// these trait bounds are copied from `impl FromRequest
	// for axum::extract::path::Path`
	T: DeserializeOwned + Send,
	S: Send + Sync,
{
	type Rejection = ApiError;

	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		match axum::extract::Path::<T>::from_request_parts(parts, state).await {
			Ok(value) => Ok(Self(value.0)),
			Err(rejection) => Err(rejection.into()),
		}
	}
}

impl From<PathRejection> for ApiError {
	fn from(err: PathRejection) -> Self {
		if err.status() >= StatusCode::INTERNAL_SERVER_ERROR {
			tracing::error!(%err, "path decode error");
			return ApiError::internal_server_error(ApiErrorCode::Route, "path decode error");
		}

		ApiError::new(err.status(), ApiErrorCode::Route, err.to_string())
	}
}

pub struct Query<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for Query<T>
where
	// these trait bounds are copied from `impl FromRequest
	// for axum::extract::path::Path`
	T: DeserializeOwned + Send,
	S: Send + Sync,
{
	type Rejection = ApiError;

	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		match axum::extract::Query::<T>::from_request_parts(parts, state).await {
			Ok(value) => Ok(Self(value.0)),
			Err(rejection) => Err(rejection.into()),
		}
	}
}

impl From<QueryRejection> for ApiError {
	fn from(err: QueryRejection) -> Self {
		if err.status() >= StatusCode::INTERNAL_SERVER_ERROR {
			tracing::error!(%err, "query decode error");
			return ApiError::internal_server_error(ApiErrorCode::Route, "query decode error");
		}

		ApiError::new(err.status(), ApiErrorCode::Route, err.to_string())
	}
}
