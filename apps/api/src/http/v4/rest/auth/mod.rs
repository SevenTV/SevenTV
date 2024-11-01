use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Extension, Router};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, UserPermission};
use shared::database::stored_event::StoredEventUserSessionData;
use shared::database::user::connection::{Platform, UserConnection};
use shared::database::user::session::UserSession;
use shared::database::user::User;
use shared::database::Id;
use shared::event::{InternalEvent, InternalEventData};

use crate::connections;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::cookies::{new_cookie, Cookies};
use crate::http::middleware::session::{Session, AUTH_COOKIE};
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

const CSRF_COOKIE: &str = "seventv-v4-login-csrf";

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/login", get(login))
		.route("/logout", get(crate::http::v3::rest::auth::logout))
		.route("/login/callback", get(login_callback))
}

fn redirect_uri(global: &Arc<Global>, platform: Platform) -> Result<url::Url, ApiError> {
	global
		.config
		.api
		.api_origin
		.join(&format!("/v4/auth/login/callback&platform={}", platform as i32))
		.map_err(|e| {
			tracing::error!(err = %e, "failed to generate redirect_uri");
			ApiError::internal_server_error(ApiErrorCode::Unknown, "failed to generate redirect_uri")
		})
}

#[derive(Debug, serde::Deserialize)]
struct LoginRequest {
	pub platform: Platform,
}

async fn login(
	State(global): State<Arc<Global>>,
	Extension(cookies): Extension<Cookies>,
	Extension(session): Extension<Session>,
	Query(query): Query<LoginRequest>,
) -> Result<Response, ApiError> {
	let req = RateLimitRequest::new(RateLimitResource::Login, &session);

	req.http(&global, async {
		// redirect to platform auth url
		let (url, scope, config) = match query.platform.into() {
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

		let csrf = Id::<()>::new().to_string();
		cookies.add(new_cookie(&global, (CSRF_COOKIE, csrf.clone())));

		let redirect_uri = redirect_uri(&global, query.platform)?;

		let redirect_url = format!(
			"{}client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
			url,
			config.client_id,
			urlencoding::encode(redirect_uri.as_str()),
			urlencoding::encode(scope),
			csrf
		);

		Ok(Redirect::to(&redirect_url))
	})
	.await
}

#[derive(Debug, serde::Deserialize)]
struct CallbackRequest {
	pub platform: Platform,
	pub code: String,
	pub state: Id<()>,
}

async fn login_callback(
	State(global): State<Arc<Global>>,
	Extension(cookies): Extension<Cookies>,
	Query(query): Query<CallbackRequest>,
) -> Result<impl IntoResponse, ApiError> {
	// validate csrf
	let csrf_cookie = cookies
		.get(CSRF_COOKIE)
		.ok_or_else(|| ApiError::bad_request(ApiErrorCode::BadRequest, "missing csrf cookie"))?;

	let csrf_payload = Id::<()>::from_str(csrf_cookie.value())
		.ok()
		.filter(|id| id.timestamp().signed_duration_since(chrono::Utc::now()).num_seconds() < 300)
		.ok_or_else(|| ApiError::bad_request(ApiErrorCode::BadRequest, "invalid csrf"))?;

	if csrf_payload != query.state {
		return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "invalid csrf"));
	}

	// exchange code for access token
	let token = connections::exchange_code(
		&global,
		query.platform,
		&query.code,
		redirect_uri(&global, query.platform)?.to_string(),
	)
	.await?;

	// query user data from platform
	let user_data = connections::get_user_data(&global, query.platform, &token.access_token).await?;

	let user = transaction(&global, |mut tx| async move {
		let user = tx
			.find_one(
				filter::filter! {
					User {
						#[query(elem_match)]
						connections: UserConnection {
							platform: query.platform,
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
					platform: query.platform,
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
							platform: query.platform,
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
								platform: query.platform,
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
				data: StoredEventUserSessionData::Create {
					platform: query.platform,
				},
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

		// create cookie
		let expiration = cookie::time::OffsetDateTime::from_unix_timestamp(user_session.expires_at.timestamp())
			.map_err(|err| {
				tracing::error!(error = %err, "failed to convert expiration to cookie time");
				ApiError::internal_server_error(ApiErrorCode::Unknown, "failed to convert expiration to cookie time")
			})
			.map_err(TransactionError::Custom)?;

		cookies.add(new_cookie(&global, (AUTH_COOKIE, token.clone())).expires(expiration));
		cookies.remove(&global, CSRF_COOKIE);

		global
			.config
			.api
			.website_origin
			.join(&format!("/auth/callback&token={}", token))
			.map_err(|e| {
				tracing::error!(err = %e, "failed to generate redirect url");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::Unknown,
					"failed to generate redirect url",
				))
			})
	})
	.await;

	match res {
		Ok(redirect_url) => Ok(Redirect::to(redirect_url.as_str())),
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
