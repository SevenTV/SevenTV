use std::fmt::Display;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Extension, Router};
use hyper::StatusCode;
use mongodb::bson::doc;
use shared::database::queries::filter;
use shared::database::role::permissions::RateLimitResource;
use shared::database::stored_event::StoredEventUserSessionData;
use shared::database::user::connection::Platform;
use shared::database::user::session::UserSession;
use shared::event::{InternalEvent, InternalEventData};

use self::login::{handle_callback as handle_login_callback, handle_login};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::extract::Query;
use crate::http::middleware::cookies::Cookies;
use crate::http::middleware::session::{Session, AUTH_COOKIE};
use crate::ratelimit::RateLimitRequest;
use crate::transactions::{with_transaction, TransactionError};

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
pub struct LoginRequest {
	pub platform: LoginRequestPlatform,
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
	)
)]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/auth.route.go#L47
async fn login(
	State(global): State<Arc<Global>>,
	Extension(cookies): Extension<Cookies>,
	Extension(session): Extension<Session>,
	Query(query): Query<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
	let req = RateLimitRequest::new(RateLimitResource::Login, &session);

	req.http(&global, async {
		let location = if query.callback {
			handle_login_callback(&global, &session, query, &cookies).await?
		} else {
			handle_login(&global, &session, query.platform.into(), &cookies)?
		};

		Ok::<_, ApiError>(Redirect::to(&location))
	})
	.await
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
) -> Result<impl IntoResponse, ApiError> {
	let global = &global;
	let res = with_transaction(global, |mut tx| async move {
		// is a new session
		if let Some(user_session) = session.user_session() {
			let user_session = tx
				.find_one_and_delete(
					filter::filter! {
						UserSession {
							#[query(rename = "_id")]
							id: user_session.id,
						}
					},
					None,
				)
				.await?;

			if let Some(user_session) = user_session {
				tx.register_event(InternalEvent {
					actor: Some(session.user().unwrap().clone()),
					session_id: Some(user_session.id),
					data: InternalEventData::UserSession {
						after: user_session,
						data: StoredEventUserSessionData::Delete,
					},
					timestamp: chrono::Utc::now(),
				})?;
			}
		}

		cookies.remove(global, AUTH_COOKIE);

		Ok(())
	})
	.await;

	match res {
		Ok(_) => Response::builder()
			.status(StatusCode::NO_CONTENT)
			.body(Body::empty())
			.map_err(|err| {
				tracing::error!(error = %err, "failed to create response");
				ApiError::internal_server_error(ApiErrorCode::Auth, "failed to create response")
			}),
		Err(TransactionError::Custom(e)) => Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			Err(ApiError::internal_server_error(ApiErrorCode::Auth, "transaction failed"))
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
	// TODO: decide what to do here
	Ok(ApiError::not_implemented(ApiErrorCode::Auth, "not implemented"))
}
