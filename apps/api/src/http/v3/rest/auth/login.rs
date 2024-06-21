use std::sync::Arc;

use hyper::StatusCode;
use mongodb::bson::{doc, to_bson};
use shared::database::{Collection, Platform, User, UserConnection, UserConnectionId, UserId, UserPermission, UserSession};

use super::LoginRequest;
use crate::connections;
use crate::dataloader::user_loader::load_user_and_permissions;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AUTH_COOKIE;
use crate::http::middleware::cookies::{new_cookie, Cookies};
use crate::jwt::{AuthJwtPayload, CsrfJwtPayload, JwtState};

const CSRF_COOKIE: &str = "seventv-csrf";

const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize?";
const TWITCH_AUTH_SCOPE: &str = "";

const DISCORD_AUTH_URL: &str = "https://discord.com/oauth2/authorize?";
const DISCORD_AUTH_SCOPE: &str = "identify";

const GOOGLE_AUTH_URL: &str =
	"https://accounts.google.com/o/oauth2/v2/auth?access_type=offline&include_granted_scopes=true&";
const GOOGLE_AUTH_SCOPE: &str = "https://www.googleapis.com/auth/youtube.readonly";

pub async fn handle_callback(global: &Arc<Global>, query: LoginRequest, cookies: &Cookies) -> Result<String, ApiError> {
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

	// exchange code for access token
	let token = connections::exchange_code(
		global,
		query.platform.into(),
		&code,
		format!(
			"{}/v3/auth?callback=true&platform={}",
			global.config().api.base_url,
			query.platform
		),
	)
	.await?;

	// query user data from platform
	let user_data = connections::get_user_data(global, query.platform.into(), &token.access_token).await?;

	let user_connection = global
		.user_connection_by_platform_id_loader()
		.load(user_data.id.clone())
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

	let mut session = global.mongo().start_session(None).await.map_err(|err| {
		tracing::error!(error = %err, "failed to start session");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	session.start_transaction(None).await.map_err(|err| {
		tracing::error!(error = %err, "failed to start transaction");
		ApiError::INTERNAL_SERVER_ERROR
	})?;
	
	let user_id = match (user_connection, csrf_payload.user_id) {
		(Some(user_connection), Some(user_id)) if user_connection.user_id != user_id => {
			return Err(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"connection already paired with another user",
			));
		},
		(None, None) => {
			// create new user

			let user = User::default();

			User::collection(global.db())
				.insert_one_with_session(&user, None, &mut session)
				.await
				.map_err(|err| {
					tracing::error!(error = %err, "failed to insert user");
					ApiError::INTERNAL_SERVER_ERROR
				})?;

			user.id
		},
		(user_connection, user_id) => {
			if let Some(user_connection) = &user_connection {
				if !user_connection.allow_login {
					return Err(ApiError::new_const(StatusCode::FORBIDDEN, "not allowed to login"));
				}
			}

			// not both can be none because of the match arm above
			let user_id = user_connection.map(|c| c.user_id).or(user_id).unwrap();

			let (user, perms) = load_user_and_permissions(global, user_id)
				.await?
				.ok_or(ApiError::new_const(StatusCode::BAD_REQUEST, "user not found"))?;

			// if !perms.has(UserPermission::Login) {
			// 	return Err(ApiError::new_const(StatusCode::FORBIDDEN, "not allowed to login"));
			// }

			user.id
		},
	};

	let platform: Platform = query.platform.into();
	let platform = to_bson(&platform).expect("failed to convert platform to bson");

	UserConnection::collection(global.db())
		.update_one_with_session(
			doc! {
				"platform": platform.clone(),
				"platform_id": &user_data.id,
			},
			doc! {
				"$set": {
					"platform_username": &user_data.username,
					"platform_display_name": &user_data.display_name,
					"platform_avatar_url": &user_data.avatar,
				},
				"$setOnInsert": {
					"id": UserConnectionId::new(),
					"user_id": user_id,
					"main_connection": csrf_payload.user_id.is_none(),
					"platform": platform,
					"platform_id": user_data.id,
					"allow_login": true,
				},
			},
			Some(
				mongodb::options::UpdateOptions::builder()
					.upsert(true)
					.build(),
			),
			&mut session,
		)
		.await
		.map_err(|err| {
			tracing::error!(error = %err, "failed to update user connection");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

	let user_session = if csrf_payload.user_id.is_none() {
		let user_session = UserSession {
			user_id: user_id,
			// TODO maybe allow for this to be configurable
			expires_at: chrono::Utc::now() + chrono::Duration::days(30),
			last_used_at: chrono::Utc::now(),
			..Default::default()
		};

		UserSession::collection(global.db())
			.insert_one_with_session(&user_session, None, &mut session)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to insert user session");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		Some(user_session)
	} else {
		None
	};

	session.commit_transaction().await.map_err(|err| {
		tracing::error!(error = %err, "failed to commit transaction");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	// create jwt access token
	let redirect_url = if let Some(user_session) = user_session {
		let jwt = AuthJwtPayload::from(user_session.clone());
		let token = jwt.serialize(global).ok_or_else(|| {
			tracing::error!("failed to serialize jwt");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		// create cookie
		let expiration =
			cookie::time::OffsetDateTime::from_unix_timestamp(user_session.expires_at.timestamp()).map_err(|err| {
				tracing::error!(error = %err, "failed to convert expiration to cookie time");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		cookies.add(new_cookie(global, (AUTH_COOKIE, token.clone())).expires(expiration));

		format!(
			"{}?platform={}&token={}",
			global.config().api.connections.callback_url,
			query.platform,
			token
		)
	} else {
		format!("{}?platform={}", global.config().api.connections.callback_url, query.platform)
	};

	Ok(redirect_url)
}

pub fn handle_login(
	global: &Arc<Global>,
	user_id: Option<UserId>,
	platform: Platform,
	cookies: &Cookies,
) -> Result<String, ApiError> {
	// redirect to platform auth url
	let (url, scope, config) = match platform {
		Platform::Twitch => (TWITCH_AUTH_URL, TWITCH_AUTH_SCOPE, &global.config().api.connections.twitch),
		Platform::Discord => (DISCORD_AUTH_URL, DISCORD_AUTH_SCOPE, &global.config().api.connections.discord),
		Platform::Google => (GOOGLE_AUTH_URL, GOOGLE_AUTH_SCOPE, &global.config().api.connections.google),
		_ => return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "unsupported platform")),
	};

	let csrf = CsrfJwtPayload::new(user_id);

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
			global.config().api.base_url,
			platform
		)),
		urlencoding::encode(scope),
		csrf.random()
	);

	Ok(redirect_url)
}
