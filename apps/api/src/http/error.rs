use std::borrow::Cow;
use std::sync::Arc;

use async_graphql::ErrorExtensionValues;
use axum::response::IntoResponse;
use axum::Json;
use hyper::{HeaderMap, StatusCode};
use scuffle_metrics::metrics;

#[metrics]
mod error {
	use scuffle_metrics::CounterU64;

	pub fn constructed(status: &'static str, status_code: String) -> CounterU64;
}

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
			Self::Unknown => "UNKNOWN",
			Self::LoginRequired => "LOGIN_REQUIRED",
			Self::TransactionError => "TRANSACTION_ERROR",
			Self::MissingContext => "MISSING_CONTEXT",
			Self::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
			Self::StripeError => "STRIPE_ERROR",
			Self::PaypalError => "PAYPAL_ERROR",
			Self::BadRequest => "BAD_REQUEST",
			Self::MutationError => "MUTATION_ERROR",
			Self::LoadError => "LOAD_ERROR",
			Self::LackingPrivileges => "LACKING_PRIVILEGES",
			Self::ImageProcessorError => "IMAGE_PROCESSOR_ERROR",
		}
	}
}

impl ApiError {
	pub fn new(status_code: StatusCode, error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		error::constructed(error_code.as_str(), status_code.to_string()).incr();

		Self {
			status_code,
			error_code,
			error: error.into(),
			status: status_code.canonical_reason().unwrap_or("unknown status code").into(),
			extra_headers: None,
		}
	}

	pub fn conflict(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self::new(StatusCode::CONFLICT, error_code, error)
	}

	pub fn internal_server_error(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self::new(StatusCode::INTERNAL_SERVER_ERROR, error_code, error)
	}

	pub fn bad_request(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self::new(StatusCode::BAD_REQUEST, error_code, error)
	}

	pub fn not_found(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self::new(StatusCode::NOT_FOUND, error_code, error)
	}

	pub fn not_implemented(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self::new(StatusCode::NOT_IMPLEMENTED, error_code, error)
	}

	pub fn unauthorized(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self::new(StatusCode::UNAUTHORIZED, error_code, error)
	}

	pub fn forbidden(error_code: ApiErrorCode, error: impl Into<Cow<'static, str>>) -> Self {
		Self::new(StatusCode::FORBIDDEN, error_code, error)
	}

	pub fn too_many_requests(error: impl Into<Cow<'static, str>>) -> Self {
		Self::new(StatusCode::TOO_MANY_REQUESTS, ApiErrorCode::RateLimitExceeded, error)
	}

	pub fn with_extra_headers(mut self, headers: HeaderMap) -> Self {
		self.extra_headers = Some(Box::new(headers));
		self
	}
}

impl IntoResponse for ApiError {
	fn into_response(mut self) -> axum::http::Response<axum::body::Body> {
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
		extensions.set("status", value.status_code.as_u16());

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
