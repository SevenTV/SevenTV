use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Router};
use hyper::StatusCode;
use mongodb::bson::doc;
use shared::database::{Collection, Platform, UserSession};

use self::login::{handle_callback as handle_login_callback, handle_login};
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::extract::Query;
use crate::http::middleware::auth::AUTH_COOKIE;
use crate::http::middleware::cookies::Cookies;

mod login;

#[derive(utoipa::OpenApi)]
#[openapi(paths(login, logout, manual))]
pub struct Docs;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/", get(login))
		.route("/logout", post(logout))
		.route("/manual", get(manual))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
	pub platform: Platform,
	#[serde(default)]
	pub callback: bool,
	#[serde(default)]
	pub code: Option<String>,
	#[serde(default)]
	pub state: Option<String>,
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
		session = session.as_ref().map(|s| s.user_id.to_string())
	)
)]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/auth.route.go#L47
async fn login(
	State(global): State<Arc<Global>>,
	Extension(cookies): Extension<Cookies>,
	session: Option<Extension<UserSession>>,
	Query(query): Query<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
	let location = if query.callback {
		handle_login_callback(&global, query, &cookies).await?
	} else {
		handle_login(&global, session.as_deref(), query.platform, &cookies)?
	};

	Response::builder()
		.header(hyper::header::LOCATION, location)
		.status(StatusCode::SEE_OTHER)
		.body(Body::empty())
		.map_err(|err| {
			tracing::error!(error = %err, "failed to create response");
			ApiError::INTERNAL_SERVER_ERROR
		})
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
	session: Option<Extension<UserSession>>,
) -> Result<impl IntoResponse, ApiError> {
	if let Some(Extension(session)) = session {
		UserSession::collection(global.db())
			.delete_one(
				doc! {
					"_id": session.id,
				},
				None,
			)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to delete session");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		cookies.remove(AUTH_COOKIE);
	}

	Response::builder()
		.status(StatusCode::NO_CONTENT)
		.body(Body::empty())
		.map_err(|err| {
			tracing::error!(error = %err, "failed to create response");
			ApiError::INTERNAL_SERVER_ERROR
		})
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
	Ok(ApiError::NOT_IMPLEMENTED)
}
