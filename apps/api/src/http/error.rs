use std::borrow::Cow;

use axum::response::{IntoResponse, Response};
use axum::Json;
use hyper::StatusCode;

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ApiError {
	#[serde(skip)]
	pub status_code: StatusCode,
	pub status: Cow<'static, str>,
	pub error_code: u16,
	pub error: Cow<'static, str>,
}

impl ApiError {
	pub const BAD_REQUEST: Self = Self::new_const(StatusCode::BAD_REQUEST, "bad request");
	pub const GONE: Self = Self::new_const(StatusCode::GONE, "the requested resource is no longer available");
	pub const INTERNAL_SERVER_ERROR: Self = Self::new_const(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
	pub const NOT_FOUND: Self = Self::new_const(StatusCode::NOT_FOUND, "not found");
	pub const NOT_IMPLEMENTED: Self = Self::new_const(StatusCode::NOT_IMPLEMENTED, "not implemented");
	pub const UNAUTHORIZED: Self = Self::new_const(StatusCode::UNAUTHORIZED, "unauthorized");

	pub fn new(status_code: StatusCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self {
			status_code,
			error: error.into(),
			status: status_code.canonical_reason().unwrap_or("unknown status code").into(),
			error_code: 0,
		}
	}

	pub const fn new_const(status_code: StatusCode, error: &'static str) -> Self {
		Self {
			status_code,
			status: Cow::Borrowed("unknown"),
			error: Cow::Borrowed(error),
			error_code: 0,
		}
	}
}

impl IntoResponse for ApiError {
	fn into_response(self) -> axum::http::Response<axum::body::Body> {
		// tracing::Span::current().set_status(Status::Error {
		// 	message: Some(self.message.clone()),
		// });

		(self.status_code, Json(self)).into_response()
	}
}

pub enum EitherApiError<S> {
	Other(S),
	Api(ApiError),
}

pub fn map_result<E>(res: Result<Response, EitherApiError<E>>) -> Result<Response, E> {
	match res {
		Ok(res) => Ok(res),
		Err(EitherApiError::Other(err)) => Err(err),
		Err(EitherApiError::Api(err)) => Ok(err.into_response()),
	}
}

impl<S> From<ApiError> for EitherApiError<S> {
	fn from(err: ApiError) -> Self {
		Self::Api(err)
	}
}
