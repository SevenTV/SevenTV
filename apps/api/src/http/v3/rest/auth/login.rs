use std::sync::Arc;

use hyper::StatusCode;
use mongodb::bson::{doc, to_bson};
use shared::database::{Collection, Platform, User, UserConnection, UserId, UserPermission, UserSession};

use super::LoginRequest;
use crate::connections;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AUTH_COOKIE;
use crate::http::middleware::cookies::{new_cookie, Cookies};
use crate::jwt::{AuthJwtPayload, CsrfJwtPayload, JwtState};
use crate::dataloader::user_loader::load_user_and_permissions;

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
		query.platform,
		&code,
		format!(
			"{}/v3/auth?callback=true&platform={}",
			global.config().api.base_url,
			query.platform
		),
	)
	.await?;

	// query user data from platform
	let user_data = connections::get_user_data(global, query.platform, &token.access_token).await?;

	let mut session = global.mongo().start_session(None).await.map_err(|err| {
		tracing::error!(error = %err, "failed to start session");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	session.start_transaction(None).await.map_err(|err| {
		tracing::error!(error = %err, "failed to start transaction");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	let user = if let Some(user_id) = csrf_payload.user_id {
		let (user, perms) = load_user_and_permissions(global, user_id)
			.await?
			.ok_or(ApiError::new_const(StatusCode::BAD_REQUEST, "user not found"))?;

		if !perms.has(UserPermission::Login) {
			return Err(ApiError::new_const(StatusCode::FORBIDDEN, "not allowed to login"));
		}

		user
	} else {
		let user = User::default();

		User::collection(global.db())
			.insert_one_with_session(&user, None, &mut session)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to insert user");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		user
	};

	let connection = UserConnection::collection(global.db())
		.find_one_and_update_with_session(
			doc! {
				"platform": to_bson(&query.platform).expect("failed to convert platform to bson"),
				"platform_id": &user_data.id,
			},
			doc! {
				"$set": {
					"platform_username": &user_data.username,
					"platform_display_name": &user_data.display_name,
					"platform_avatar_url": &user_data.avatar,
					"updated_at": chrono::Utc::now(),
				},
				"$setOnInsert": to_bson(&UserConnection {
					user_id: user.id,
					main_connection: csrf_payload.user_id.is_none(),
					platform: query.platform,
					platform_id: user_data.id,
					platform_username: user_data.username,
					platform_display_name: user_data.display_name,
					platform_avatar_url: user_data.avatar,
					allow_login: true,
					..Default::default()
				}).expect("failed to convert user connection to bson"),
			},
			Some(
				mongodb::options::FindOneAndUpdateOptions::builder()
					.upsert(true)
					.return_document(mongodb::options::ReturnDocument::After)
					.build(),
			),
			&mut session,
		)
		.await
		.map_err(|err| {
			tracing::error!(error = %err, "failed to find user connection");
			ApiError::INTERNAL_SERVER_ERROR
		})?
		.ok_or_else(|| {
			tracing::error!("user connection failed to be created");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

	if connection.user_id != user.id {
		return Err(ApiError::new_const(
			StatusCode::BAD_REQUEST,
			"connection already paired with another user",
		));
	}

	if connection.allow_login && csrf_payload.user_id.is_none() {
		return Err(ApiError::new_const(
			StatusCode::UNAUTHORIZED,
			"connection is not allowed to login",
		));
	}

	let user_session = if csrf_payload.user_id.is_none() {
		let user_session = UserSession {
			user_id: user.id,
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
