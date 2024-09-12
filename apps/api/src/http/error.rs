use std::borrow::Cow;
use std::sync::Arc;

use async_graphql::ErrorExtensionValues;
use axum::response::IntoResponse;
use axum::Json;
use hyper::{HeaderMap, StatusCode};

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ApiError {
	#[serde(skip)]
	pub status_code: StatusCode,
	pub status: Cow<'static, str>,
	pub error_code: ApiErrorCode,
	pub error: Cow<'static, str>,
	#[serde(skip)]
	pub extra_headers: Option<Box<HeaderMap>>,
}

#[derive(Debug, Default, Clone, Copy, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(u16)]
pub enum ApiErrorCode {
	#[default]
	Unknown = 0,

	/// Login Required
	LoginRequired = 1000,
	/// Transaction Error
	TransactionError = 5000,
	/// Missing Application Context
	MissingContext = 6000,
	/// Rate Limit
	RateLimitExceeded = 7000,
	/// Stripe Error
	StripeError = 8000,
	/// Paypal Error
	PaypalError = 9000,
	/// Bad Request
	BadRequest = 10000,
	/// Mutation Error
	MutationError = 11000,
	/// Load Error
	LoadError = 12000,
	/// Lacking Privileges
	LackingPrivileges = 20000,
	/// Image Processor Error
	ImageProcessorError = 21000,
}

impl ApiErrorCode {
	pub fn as_str(&self) -> &'static str {
		match self {
			_ => "UNKNOWN",
		}
	}
}

impl ApiError {
	pub fn new(status_code: StatusCode, error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self {
			status_code,
			error_code,
			error: error.into(),
			status: status_code.canonical_reason().unwrap_or("unknown status code").into(),
			extra_headers: None,
		}
	}

	pub fn conflict(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self {
			status_code: StatusCode::CONFLICT,
			error_code,
			error: error.into(),
			status: Cow::Borrowed("conflict"),
			extra_headers: None,
		}
	}

	pub fn internal_server_error(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self {
			status_code: StatusCode::INTERNAL_SERVER_ERROR,
			error_code,
			error: error.into(),
			status: Cow::Borrowed("internal server error"),
			extra_headers: None,
		}
	}

	pub fn bad_request(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self {
			status_code: StatusCode::BAD_REQUEST,
			error_code,
			error: error.into(),
			status: Cow::Borrowed("bad request"),
			extra_headers: None,
		}
	}

	pub fn not_found(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self {
			status_code: StatusCode::NOT_FOUND,
			error_code,
			error: error.into(),
			status: Cow::Borrowed("not found"),
			extra_headers: None,
		}
	}

	pub fn not_implemented(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self {
			status_code: StatusCode::NOT_IMPLEMENTED,
			error_code,
			error: error.into(),
			status: Cow::Borrowed("not implemented"),
			extra_headers: None,
		}
	}

	pub fn unauthorized(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self {
			status_code: StatusCode::UNAUTHORIZED,
			error_code,
			error: error.into(),
			status: Cow::Borrowed("unauthorized"),
			extra_headers: None,
		}
	}

	pub fn forbidden(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self {
			status_code: StatusCode::FORBIDDEN,
			error_code,
			error: error.into(),
			status: Cow::Borrowed("forbidden"),
			extra_headers: None,
		}
	}

	pub fn too_many_requests(error: impl Into<Cow<'static, str>>) -> Self {
		Self {
			status_code: StatusCode::TOO_MANY_REQUESTS,
			error_code: ApiErrorCode::RateLimitExceeded,
			error: error.into(),
			status: Cow::Borrowed("too many requests"),
			extra_headers: None,
		}
	}

	pub fn with_extra_headers(mut self, headers: HeaderMap) -> Self {
		self.extra_headers = Some(Box::new(headers));
		self
	}
}

impl IntoResponse for ApiError {
	fn into_response(mut self) -> axum::http::Response<axum::body::Body> {
		// tracing::Span::current().set_status(Status::Error {
		// 	message: Some(self.message.clone()),
		// });

		let extra_headers = self.extra_headers.take();

		let mut resp = (self.status_code, Json(self)).into_response();

		if let Some(headers) = extra_headers {
			resp.headers_mut().extend(*headers);
		}

		resp
	}
}

impl From<ApiError> for async_graphql::Error {
	fn from(value: ApiError) -> Self {
		let mut extensions = ErrorExtensionValues::default();
		extensions.set("code", value.error_code.as_str());
		// for backward compatibility
		extensions.set("fields", async_graphql::Value::Object(Default::default()));
		// The old website expects the error message to be in the format "title:
		// description"
		let message = format!("{} {}", value.error_code.as_str(), value.error);
		extensions.set("message", message.clone());
		if let Some(headers) = &value.extra_headers {
			extensions.set(
				"headers",
				async_graphql::Value::Object(
					headers
						.iter()
						.map(|(k, v)| {
							(
								async_graphql::Name::new(k.as_str()),
								async_graphql::Value::String(v.to_str().unwrap_or_default().to_string()),
							)
						})
						.collect(),
				),
			);
		}

		Self {
			message,
			source: Some(Arc::new(value)),
			extensions: Some(extensions),
		}
	}
}
