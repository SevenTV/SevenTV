use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Extension, Router};
use hyper::StatusCode;
use mongodb::bson::doc;
use shared::database::role::permissions::RateLimitResource;
use shared::database::user::connection::Platform;

use self::login::{handle_callback as handle_login_callback, handle_login};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::extract::Query;
use crate::http::middleware::cookies::Cookies;
use crate::http::middleware::session::{parse_session, Session, AUTH_COOKIE};
use crate::ratelimit::RateLimitRequest;
use crate::transactions::TransactionError;

mod login;

#[derive(utoipa::OpenApi)]
#[openapi(paths(login, logout, manual))]
pub struct Docs;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/", get(login))
		.route("/logout", get(logout))
		.route("/manual", get(manual))
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LoginRequestPlatform {
	Twitch,
	Discord,
	Youtube,
}

impl From<LoginRequestPlatform> for Platform {
	fn from(platform: LoginRequestPlatform) -> Self {
		match platform {
			LoginRequestPlatform::Twitch => Platform::Twitch,
			LoginRequestPlatform::Discord => Platform::Discord,
			LoginRequestPlatform::Youtube => Platform::Google,
		}
	}
}

impl Display for LoginRequestPlatform {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			LoginRequestPlatform::Twitch => write!(f, "twitch"),
			LoginRequestPlatform::Discord => write!(f, "discord"),
			LoginRequestPlatform::Youtube => write!(f, "youtube"),
		}
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
struct LoginRequest {
	pub platform: LoginRequestPlatform,
	#[serde(default)]
	pub link_connection: bool,
	#[serde(default)]
	pub callback: bool,
	#[serde(default)]
	pub code: Option<String>,
	#[serde(default)]
	pub state: Option<String>,
	#[serde(default)]
	pub token: Option<String>,
}

#[utoipa::path(
    get,
    path = "/v3/auth",
    tag = "auth",
    responses(
        (status = 303, description = "Auth Redirect"),
    ),
)]
#[tracing::instrument(
	skip_all,
	fields(
		query.platform = %query.platform,
		query.callback = %query.callback,
	)
)]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/auth.route.go#L47
async fn login(
	State(global): State<Arc<Global>>,
	Extension(cookies): Extension<Cookies>,
	Extension(session): Extension<Session>,
	Query(query): Query<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
	let session = if let (Some(token), None) = (&query.token, session.user_session()) {
		parse_session(&global, session.ip(), token)
			.await?
			.ok_or_else(|| ApiError::unauthorized(ApiErrorCode::BadRequest, "invalid token"))?
	} else {
		session
	};

	let req = RateLimitRequest::new(RateLimitResource::Login, &session);

	req.http(&global, async {
		let location = if query.callback {
			handle_login_callback(&global, &session, query, &cookies).await?
		} else {
			handle_login(&global, &session, query.platform.into(), query.link_connection, &cookies)?
		};

		Ok::<_, ApiError>(Redirect::to(&location))
	})
	.await
}

#[derive(Debug, serde::Deserialize)]
struct LogoutRequest {
	#[serde(default)]
	pub token: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v3/auth/logout",
    tag = "auth",
    responses(
        (status = 204, description = "Logout"),
    ),
)]
#[tracing::instrument(skip(global, cookies))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/logout.auth.route.go#L29
async fn logout(
	State(global): State<Arc<Global>>,
	Extension(cookies): Extension<Cookies>,
	Extension(session): Extension<Session>,
	Query(query): Query<LogoutRequest>,
	request: axum::extract::Request,
) -> Result<impl IntoResponse, ApiError> {
	let allowed = [
		&global.config.api.api_origin,
		&global.config.api.website_origin,
		&global.config.api.beta_website_origin,
	];

	if let Some(referer) = request.headers().get(hyper::header::REFERER) {
		let referer = referer.to_str().ok().map(|s| url::Url::from_str(s).ok()).flatten();
		if !referer.is_some_and(|u| allowed.iter().any(|a| u.origin() == a.origin())) {
			return Err(ApiError::forbidden(ApiErrorCode::BadRequest, "can only logout from website"));
		}
	}

	if let Some(origin) = request.headers().get(hyper::header::ORIGIN) {
		let origin = origin.to_str().ok().map(|s| url::Url::from_str(s).ok()).flatten();
		if !origin.is_some_and(|u| allowed.iter().any(|a| u.origin() == a.origin())) {
			return Err(ApiError::forbidden(ApiErrorCode::BadRequest, "origin mismatch"));
		}
	}

	let session = if session.user_session().is_none() {
		if let Some(token) = &query.token {
			parse_session(&global, session.ip(), token)
				.await?
				.ok_or_else(|| ApiError::unauthorized(ApiErrorCode::BadRequest, "invalid token"))?
		} else {
			session
		}
	} else {
		session
	};

	match session.logout(&global).await {
		Ok(_) => {
			cookies.remove(&global, AUTH_COOKIE);

			let website_url = global
				.config
				.api
				.website_origin
				.join("/auth/callback#logout=true")
				.map_err(|err| {
					tracing::error!(error = %err, "failed to join website origin");
					ApiError::internal_server_error(ApiErrorCode::Unknown, "failed to join website origin")
				})?;

			Response::builder()
				.status(StatusCode::TEMPORARY_REDIRECT)
				.header("Location", website_url.to_string())
				.body(Body::empty())
				.map_err(|err| {
					tracing::error!(error = %err, "failed to create response");
					ApiError::internal_server_error(ApiErrorCode::Unknown, "failed to create response")
				})
		}
		Err(TransactionError::Custom(e)) => Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			Err(ApiError::internal_server_error(
				ApiErrorCode::TransactionError,
				"transaction failed",
			))
		}
	}
}

#[utoipa::path(
    get,
    path = "/v3/auth/manual",
    tag = "auth",
    responses(
        (status = 200, description = "Manual Auth"),
    ),
)]
#[tracing::instrument]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/manual.route.go#L41
async fn manual() -> Result<impl IntoResponse, ApiError> {
	// won't be implemented
	Ok(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
}
