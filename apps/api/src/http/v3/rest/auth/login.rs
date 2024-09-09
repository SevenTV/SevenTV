use std::sync::Arc;

use hyper::StatusCode;
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{PermissionsExt, UserPermission};
use shared::database::stored_event::StoredEventUserSessionData;
use shared::database::user::connection::{Platform, UserConnection};
use shared::database::user::session::UserSession;
use shared::database::user::User;
use shared::event::{InternalEvent, InternalEventData};

use super::LoginRequest;
use crate::connections;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::cookies::{new_cookie, Cookies};
use crate::http::middleware::session::{Session, AUTH_COOKIE};
use crate::jwt::{AuthJwtPayload, CsrfJwtPayload, JwtState};
use crate::transactions::{with_transaction, TransactionError};

const CSRF_COOKIE: &str = "seventv-csrf";

const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize?";
const TWITCH_AUTH_SCOPE: &str = "";

const DISCORD_AUTH_URL: &str = "https://discord.com/oauth2/authorize?";
const DISCORD_AUTH_SCOPE: &str = "identify";

const GOOGLE_AUTH_URL: &str =
	"https://accounts.google.com/o/oauth2/v2/auth?access_type=offline&include_granted_scopes=true&";
const GOOGLE_AUTH_SCOPE: &str = "https://www.googleapis.com/auth/youtube.readonly";

/// https://gist.github.com/lennartkloock/412323105bc913c7064664dc4f1568cb
pub async fn handle_callback(
	global: &Arc<Global>,
	_session: &Session,
	query: LoginRequest,
	cookies: &Cookies,
) -> Result<String, ApiError> {
	let code = query
		.code
		.ok_or(ApiError::new_const(StatusCode::BAD_REQUEST, "missing code from query"))?;
	let state = query
		.state
		.ok_or(ApiError::new_const(StatusCode::BAD_REQUEST, "missing state from query"))?;

	// validate csrf
	let csrf_cookie = cookies
		.get(CSRF_COOKIE)
		.ok_or(ApiError::new_const(StatusCode::BAD_REQUEST, "missing csrf cookie"))?;

	let csrf_payload = CsrfJwtPayload::verify(global, csrf_cookie.value())
		.filter(|payload| payload.validate_random(&state).unwrap_or_default())
		.ok_or(ApiError::new_const(StatusCode::BAD_REQUEST, "invalid csrf"))?;

	let platform = Platform::from(query.platform);

	// exchange code for access token
	let token = connections::exchange_code(
		global,
		platform,
		&code,
		format!(
			"{}/v3/auth?callback=true&platform={}",
			global.config.api.api_origin, query.platform
		),
	)
	.await?;

	// query user data from platform
	let user_data = connections::get_user_data(global, platform, &token.access_token).await?;

	let user = with_transaction(global, |mut tx| async move {
		let user = tx
			.find_one(
				filter::filter! {
					User {
						#[query(elem_match)]
						connections: UserConnection {
							platform,
							platform_id: &user_data.id,
						}
					}
				},
				None,
			)
			.await?;

		let user_id = match (csrf_payload.user_id, user) {
			// user tries to link a different account
			(Some(user_id), Some(user)) if user_id != user.id => {
				// deny log in
				return Err(TransactionError::custom(ApiError::new_const(
					StatusCode::BAD_REQUEST,
					"connection already paired with another user",
				)));
			}
			// user links an already linked account
			// we know that (user_id == user.id) is true here
			(Some(user_id), Some(_)) => user_id,
			// user links a new account
			(Some(user_id), None) => user_id,
			// user logs in with an existing account
			(None, Some(user)) => {
				let connection = user
					.connections
					.iter()
					.find(|c| c.platform_id == user_data.id)
					.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

				if !connection.allow_login {
					return Err(TransactionError::custom(ApiError::new_const(
						StatusCode::UNAUTHORIZED,
						"connection is not allowed to login",
					)));
				}

				user.id
			}
			// user logs in for the first time
			(None, None) => {
				// create new user
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

				user.id
			}
		};

		// upsert the connection
		tx.find_one_and_update(
			filter::filter! {
				User {
					#[query(rename = "_id")]
					id: user_id,
					#[query(elem_match)]
					connections: UserConnection {
						platform,
						platform_id: &user_data.id,
					}
				}
			},
			update::update! {
				#[query(set)]
				User {
					#[query(flatten, index = "$")]
					connections: UserConnection {
						platform_username: user_data.username,
						platform_display_name: user_data.display_name,
						platform_avatar_url: user_data.avatar,
						updated_at: chrono::Utc::now(),
					},
					updated_at: chrono::Utc::now(),
				}
			},
			None,
		)
		.await?
		.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))
	})
	.await;

	let user = match user {
		Ok(user) => user,
		Err(TransactionError::Custom(e)) => return Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			return Err(ApiError::INTERNAL_SERVER_ERROR);
		}
	};

	let full_user = global
		.user_loader
		.load_user(global, user)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

	if !full_user.has(UserPermission::Login) {
		return Err(ApiError::new_const(StatusCode::FORBIDDEN, "not allowed to login"));
	}

	let res = with_transaction(global, |mut tx| async move {
		if csrf_payload.user_id.is_none() {
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
				.serialize(global)
				.ok_or_else(|| {
					tracing::error!("failed to serialize jwt");
					ApiError::INTERNAL_SERVER_ERROR
				})
				.map_err(TransactionError::custom)?;

			// create cookie
			let expiration = cookie::time::OffsetDateTime::from_unix_timestamp(user_session.expires_at.timestamp())
				.map_err(|err| {
					tracing::error!(error = %err, "failed to convert expiration to cookie time");
					ApiError::INTERNAL_SERVER_ERROR
				})
				.map_err(TransactionError::custom)?;

			cookies.add(new_cookie(global, (AUTH_COOKIE, token.clone())).expires(expiration));
			cookies.remove(global, CSRF_COOKIE);

			Ok(format!(
				"{}/auth/callback?platform={}&token={}",
				global.config.api.website_origin, query.platform, token
			))
		} else {
			Ok(format!(
				"{}/auth/callback?platform={}",
				global.config.api.website_origin, query.platform
			))
		}
	})
	.await;

	match res {
		Ok(redirect_url) => Ok(redirect_url),
		Err(TransactionError::Custom(e)) => Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			Err(ApiError::INTERNAL_SERVER_ERROR)
		}
	}
}

pub fn handle_login(
	global: &Arc<Global>,
	session: &Session,
	platform: Platform,
	cookies: &Cookies,
) -> Result<String, ApiError> {
	// redirect to platform auth url
	let (url, scope, config) = match platform {
		Platform::Twitch if global.config.api.connections.twitch.enabled => {
			(TWITCH_AUTH_URL, TWITCH_AUTH_SCOPE, &global.config.api.connections.twitch)
		}
		Platform::Discord if global.config.api.connections.discord.enabled => {
			(DISCORD_AUTH_URL, DISCORD_AUTH_SCOPE, &global.config.api.connections.discord)
		}
		Platform::Google if global.config.api.connections.google.enabled => {
			(GOOGLE_AUTH_URL, GOOGLE_AUTH_SCOPE, &global.config.api.connections.google)
		}
		_ => {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "unsupported platform"));
		}
	};

	let csrf = CsrfJwtPayload::new(session);

	cookies.add(new_cookie(
		global,
		(
			CSRF_COOKIE,
			csrf.serialize(global).ok_or_else(|| {
				tracing::error!("failed to serialize csrf");
				ApiError::INTERNAL_SERVER_ERROR
			})?,
		),
	));

	let redirect_url = format!(
		"{}client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
		url,
		config.client_id,
		urlencoding::encode(&format!(
			"{}/v3/auth?callback=true&platform={}",
			global.config.api.api_origin, platform
		)),
		urlencoding::encode(scope),
		csrf.random()
	);

	Ok(redirect_url)
}
