use std::borrow::Cow;
use std::sync::Arc;

use async_graphql::ErrorExtensionValues;
use axum::response::IntoResponse;
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
	pub const FORBIDDEN: Self = Self::new_const(StatusCode::FORBIDDEN, "forbidden");
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

impl From<ApiError> for async_graphql::Error {
	fn from(value: ApiError) -> Self {
		let mut extensions = ErrorExtensionValues::default();
		extensions.set("code", value.error_code);
		// for backward compatibility
		extensions.set("fields", async_graphql::Value::Object(Default::default()));
		// The old website expects the error message to be in the format "title: description"
		let message = format!("Error: {}", value.error);
		extensions.set("message", message.clone());

		Self {
			message,
			source: Some(Arc::new(value)),
			extensions: Some(extensions),
		}
	}
}
