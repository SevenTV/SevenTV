use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use hyper::HeaderMap;
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, UserPermission};
use shared::database::stored_event::StoredEventUserSessionData;
use shared::database::user::connection::{Platform, UserConnection};
use shared::database::user::session::UserSession;
use shared::database::user::User;
use shared::event::{InternalEvent, InternalEventData};

use crate::connections;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::cookies::Cookies;
use crate::http::middleware::session::{parse_session, Session, AUTH_COOKIE};
use crate::jwt::{AuthJwtPayload, JwtState};
use crate::ratelimit::RateLimitRequest;
use crate::transactions::{transaction, TransactionError};

const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize?";
const TWITCH_AUTH_SCOPE: &str = "";

const DISCORD_AUTH_URL: &str = "https://discord.com/oauth2/authorize?";
const DISCORD_AUTH_SCOPE: &str = "identify";

const GOOGLE_AUTH_URL: &str =
	"https://accounts.google.com/o/oauth2/v2/auth?access_type=offline&include_granted_scopes=true&";
const GOOGLE_AUTH_SCOPE: &str = "https://www.googleapis.com/auth/youtube.readonly";

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/login", get(login))
		.route("/login/finish", post(login_finish))
		.route("/logout", post(logout))
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum LoginPlatform {
	Twitch,
	Discord,
	Google,
	Kick,
}

impl From<LoginPlatform> for Platform {
	fn from(platform: LoginPlatform) -> Self {
		match platform {
			LoginPlatform::Twitch => Platform::Twitch,
			LoginPlatform::Discord => Platform::Discord,
			LoginPlatform::Google => Platform::Google,
			LoginPlatform::Kick => Platform::Kick,
		}
	}
}

impl Display for LoginPlatform {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			LoginPlatform::Twitch => write!(f, "twitch"),
			LoginPlatform::Discord => write!(f, "discord"),
			LoginPlatform::Google => write!(f, "google"),
			LoginPlatform::Kick => write!(f, "kick"),
		}
	}
}

fn redirect_uri(global: &Arc<Global>, platform: LoginPlatform) -> Result<url::Url, ApiError> {
	global
		.config
		.api
		.beta_website_origin
		.join(&format!("/login/callback?platform={}", platform))
		.map_err(|e| {
			tracing::error!(err = %e, "failed to generate redirect_uri");
			ApiError::internal_server_error(ApiErrorCode::Unknown, "failed to generate redirect_uri")
		})
}

#[derive(Debug, serde::Deserialize)]
struct LoginRequest {
	pub platform: LoginPlatform,
	pub return_to: Option<String>,
}

async fn login(
	State(global): State<Arc<Global>>,
	Extension(session): Extension<Session>,
	Query(query): Query<LoginRequest>,
	headers: HeaderMap,
) -> Result<Response, ApiError> {
	let allowed = [
		&global.config.api.api_origin,
		&global.config.api.website_origin,
		&global.config.api.beta_website_origin,
	];

	if let Some(referer) = headers.get(hyper::header::REFERER) {
		let referer = referer.to_str().ok().map(|s| url::Url::from_str(s).ok()).flatten();
		if !referer.is_some_and(|u| allowed.iter().any(|a| u.origin() == a.origin())) {
			return Err(ApiError::forbidden(ApiErrorCode::BadRequest, "can only login from website"));
		}
	}

	if let Some(origin) = headers.get(hyper::header::ORIGIN) {
		let origin = origin.to_str().ok().map(|s| url::Url::from_str(s).ok()).flatten();
		if !origin.is_some_and(|u| allowed.iter().any(|a| u.origin() == a.origin())) {
			return Err(ApiError::forbidden(ApiErrorCode::BadRequest, "origin mismatch"));
		}
	}

	if let Some(return_to) = query.return_to.as_ref().and_then(|u| url::Url::from_str(u).ok()) {
		if !allowed.iter().any(|a| return_to.origin() == a.origin()) {
			return Err(ApiError::forbidden(ApiErrorCode::BadRequest, "return_to origin mismatch"));
		}
	}

	let platform = query.platform.into();

	let req = RateLimitRequest::new(RateLimitResource::Login, &session);

	req.http(&global, async {
		// redirect to platform auth url
		let (url, scope, config) = match platform {
			Platform::Twitch if global.config.connections.twitch.enabled => {
				(TWITCH_AUTH_URL, TWITCH_AUTH_SCOPE, &global.config.connections.twitch)
			}
			Platform::Discord if global.config.connections.discord.enabled => {
				(DISCORD_AUTH_URL, DISCORD_AUTH_SCOPE, &global.config.connections.discord)
			}
			Platform::Google if global.config.connections.google.enabled => {
				(GOOGLE_AUTH_URL, GOOGLE_AUTH_SCOPE, &global.config.connections.google)
			}
			_ => {
				return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "unsupported platform"));
			}
		};

		let redirect_uri = redirect_uri(&global, query.platform)?;

		let redirect_url = format!(
			"{}client_id={}&redirect_uri={}&response_type=code&scope={}{}",
			url,
			config.client_id,
			urlencoding::encode(redirect_uri.as_str()),
			urlencoding::encode(scope),
			query
				.return_to
				.as_ref()
				.map(|r| format!("&state={}", urlencoding::encode(r.as_str())))
				.unwrap_or_default()
		);

		Ok(Redirect::to(&redirect_url))
	})
	.await
}

#[derive(Debug, serde::Deserialize)]
struct LoginFinishPayload {
	pub platform: LoginPlatform,
	pub code: String,
}

#[derive(Debug, serde::Serialize)]
struct LoginFinishResponse {
	pub token: String,
}

async fn login_finish(
	State(global): State<Arc<Global>>,
	headers: HeaderMap,
	Json(payload): Json<LoginFinishPayload>,
) -> Result<impl IntoResponse, ApiError> {
	let allowed = [
		&global.config.api.api_origin,
		&global.config.api.website_origin,
		&global.config.api.beta_website_origin,
	];

	if let Some(referer) = headers.get(hyper::header::REFERER) {
		let referer = referer.to_str().ok().map(|s| url::Url::from_str(s).ok()).flatten();
		if !referer.is_some_and(|u| allowed.iter().any(|a| u.origin() == a.origin())) {
			return Err(ApiError::forbidden(ApiErrorCode::BadRequest, "can only login from website"));
		}
	}

	if let Some(origin) = headers.get(hyper::header::ORIGIN) {
		let origin = origin.to_str().ok().map(|s| url::Url::from_str(s).ok()).flatten();
		if !origin.is_some_and(|u| allowed.iter().any(|a| u.origin() == a.origin())) {
			return Err(ApiError::forbidden(ApiErrorCode::BadRequest, "origin mismatch"));
		}
	}

	let platform = payload.platform.into();

	// exchange code for access token
	let token = connections::exchange_code(
		&global,
		platform,
		&payload.code,
		redirect_uri(&global, payload.platform)?.to_string(),
	)
	.await?;

	// query user data from platform
	let user_data = connections::get_user_data(&global, platform, &token.access_token).await?;

	let user = transaction(&global, |mut tx| async move {
		let user = tx
			.find_one(
				filter::filter! {
					User {
						#[query(elem_match)]
						connections: UserConnection {
							platform: platform,
							platform_id: &user_data.id,
						}
					}
				},
				None,
			)
			.await?;

		let Some(user) = user else {
			let user = User {
				connections: vec![UserConnection {
					platform,
					platform_id: user_data.id.clone(),
					platform_username: user_data.username.clone(),
					platform_display_name: user_data.display_name.clone(),
					platform_avatar_url: user_data.avatar.clone(),
					allow_login: true,
					updated_at: chrono::Utc::now(),
					linked_at: chrono::Utc::now(),
				}],
				..Default::default()
			};

			tx.insert_one::<User>(&user, None).await?;

			return Ok(user);
		};

		let connection = user
			.connections
			.iter()
			.find(|c| c.platform_id == user_data.id)
			.ok_or_else(|| {
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::LoadError,
					"failed to load connection",
				))
			})?;

		if !connection.allow_login {
			return Err(TransactionError::Custom(ApiError::unauthorized(
				ApiErrorCode::LackingPrivileges,
				"connection is not allowed to login",
			)));
		}

		// upsert the connection
		let updated = tx
			.find_one_and_update(
				filter::filter! {
					User {
						#[query(rename = "_id")]
						id: user.id,
						#[query(elem_match)]
						connections: UserConnection {
							platform: platform,
							platform_id: &user_data.id,
						}
					}
				},
				update::update! {
					#[query(set)]
					User {
						#[query(flatten, index = "$")]
						connections: UserConnection {
							platform_username: &user_data.username,
							platform_display_name: &user_data.display_name,
							platform_avatar_url: &user_data.avatar,
							updated_at: chrono::Utc::now(),
						},
						updated_at: chrono::Utc::now(),
						search_updated_at: &None,
					}
				},
				None,
			)
			.await?;

		let updated = match updated {
			Some(user) => user,
			None => tx
				.find_one_and_update(
					filter::filter! {
						User {
							#[query(rename = "_id")]
							id: user.id,
						}
					},
					update::update! {
						#[query(push)]
						User {
							#[query(serde)]
							connections: UserConnection {
								platform,
								platform_id: user_data.id,
								platform_username: user_data.username,
								platform_display_name: user_data.display_name,
								platform_avatar_url: user_data.avatar,
								allow_login: true,
								updated_at: chrono::Utc::now(),
								linked_at: chrono::Utc::now(),
							},
						},
						#[query(set)]
						User {
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					},
					None,
				)
				.await?
				.ok_or_else(|| {
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::MutationError,
						"failed to insert connection",
					))
				})?,
		};

		Ok(updated)
	})
	.await;

	let user = match user {
		Ok(user) => user,
		Err(TransactionError::Custom(e)) => return Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			return Err(ApiError::internal_server_error(
				ApiErrorCode::TransactionError,
				"transaction failed",
			));
		}
	};

	let full_user = global
		.user_loader
		.load_user(&global, user)
		.await
		.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

	if !full_user.has(UserPermission::Login) {
		return Err(ApiError::forbidden(ApiErrorCode::LackingPrivileges, "not allowed to login"));
	}

	let res = transaction(&Arc::clone(&global), |mut tx| async move {
		let user_session = UserSession {
			id: Default::default(),
			user_id: full_user.id,
			// TODO: maybe allow for this to be configurable
			expires_at: chrono::Utc::now() + chrono::Duration::days(30),
			last_used_at: chrono::Utc::now(),
		};

		tx.insert_one::<UserSession>(&user_session, None).await?;

		tx.register_event(InternalEvent {
			actor: Some(full_user.clone()),
			session_id: None,
			data: InternalEventData::UserSession {
				after: user_session.clone(),
				data: StoredEventUserSessionData::Create { platform },
			},
			timestamp: chrono::Utc::now(),
		})?;

		// create jwt access token
		let jwt = AuthJwtPayload::from(user_session.clone());
		let token = jwt
			.serialize(&global)
			.ok_or_else(|| {
				tracing::error!("failed to serialize jwt");
				ApiError::internal_server_error(ApiErrorCode::Unknown, "failed to serialize jwt")
			})
			.map_err(TransactionError::Custom)?;

		Ok(LoginFinishResponse { token })
	})
	.await;

	match res {
		Ok(response) => Ok(Json(response)),
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

#[derive(Debug, serde::Deserialize)]
struct LogoutRequest {
	#[serde(default)]
	pub token: Option<String>,
}

async fn logout(
	State(global): State<Arc<Global>>,
	Extension(cookies): Extension<Cookies>,
	Extension(session): Extension<Session>,
	Query(query): Query<LogoutRequest>,
	headers: HeaderMap,
) -> Result<(), ApiError> {
	let allowed = [
		&global.config.api.api_origin,
		&global.config.api.website_origin,
		&global.config.api.beta_website_origin,
	];

	if let Some(referer) = headers.get(hyper::header::REFERER) {
		let referer = referer.to_str().ok().map(|s| url::Url::from_str(s).ok()).flatten();
		if !referer.is_some_and(|u| allowed.iter().any(|a| u.origin() == a.origin())) {
			return Err(ApiError::forbidden(ApiErrorCode::BadRequest, "can only logout from website"));
		}
	}

	if let Some(origin) = headers.get(hyper::header::ORIGIN) {
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
			Ok(())
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
