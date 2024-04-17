use std::sync::Arc;

use hyper::body::Incoming;
use hyper::StatusCode;
use mongodb::bson::{doc, to_bson};
use scuffle_utils::http::ext::{OptionExt, ResultExt};
use scuffle_utils::http::RouteError;
use shared::database::{Collection, Platform, User, UserConnection, UserSession};

use crate::connections;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::{new_cookie, Cookies, AUTH_COOKIE};
use crate::http::RequestQueryParamExt;
use crate::jwt::{AuthJwtPayload, CsrfJwtPayload, JwtState};

const CSRF_COOKIE: &str = "seventv-csrf";

const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize?";
const TWITCH_AUTH_SCOPE: &str = "";

const DISCORD_AUTH_URL: &str = "https://discord.com/oauth2/authorize?";
const DISCORD_AUTH_SCOPE: &str = "identify";

const GOOGLE_AUTH_URL: &str =
	"https://accounts.google.com/o/oauth2/v2/auth?access_type=offline&include_granted_scopes=true&";
const GOOGLE_AUTH_SCOPE: &str = "https://www.googleapis.com/auth/youtube.readonly";

pub async fn handle_callback(
	global: &Arc<Global>,
	platform: Platform,
	req: &hyper::Request<Incoming>,
	cookies: &Cookies,
) -> Result<String, RouteError<ApiError>> {
	let code = req
		.query_param("code")
		.map_err_route((StatusCode::BAD_REQUEST, "missing code"))?;

	// validate csrf
	let state = req
		.query_param("state")
		.map_err_route((StatusCode::BAD_REQUEST, "missing state"))?;
	let csrf_cookie = cookies
		.get(CSRF_COOKIE)
		.map_err_route((StatusCode::BAD_REQUEST, "missing csrf cookie"))?;

	let csrf_payload = CsrfJwtPayload::verify(global, csrf_cookie.value())
		.filter(|payload| payload.validate_random(&state).unwrap_or_default())
		.map_err_route((StatusCode::BAD_REQUEST, "invalid csrf"))?;

	// exchange code for access token
	let token = connections::exchange_code(
		global,
		platform,
		&code,
		format!("{}/v3/auth?callback=true&platform={}", global.config().api.base_url, platform),
	)
	.await
	.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to exchange code"))?;

	// query user data from platform
	let user_data = connections::get_user_data(global, platform, &token.access_token)
		.await
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to get user data from platform"))?;

	let mut session = global
		.mongo()
		.start_session(None)
		.await
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to start session"))?;
	session
		.start_transaction(None)
		.await
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to start transaction"))?;

	let user = if let Some(user_id) = csrf_payload.user_id {
		User::collection(global.db())
			.find_one_with_session(
				doc! {
					"_id": user_id,
				},
				None,
				&mut session,
			)
			.await
			.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to find user"))?
			.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to find user"))?
	} else {
		let user = User::default();
		User::collection(global.db())
			.insert_one_with_session(&user, None, &mut session)
			.await
			.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to create user"))?;

		user
	};

	let connection = UserConnection::collection(global.db())
		.find_one_and_update_with_session(
			doc! {
				"platform": to_bson(&platform).expect("failed to convert platform to bson"),
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
					platform,
					platform_id: user_data.id,
					platform_username: user_data.username,
					platform_display_name: user_data.display_name,
					platform_avatar_url: user_data.avatar,
					updated_at: chrono::Utc::now(),
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
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to update user connection"))?
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to create user connection"))?;

	if connection.user_id != user.id {
		return Err((StatusCode::BAD_REQUEST, "connection already connected to another account").into());
	}

	if connection.allow_login && csrf_payload.user_id.is_none() {
		return Err((StatusCode::BAD_REQUEST, "connection has login disabled").into());
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
			.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to create user session"))?;

		Some(user_session)
	} else {
		None
	};

	session
		.commit_transaction()
		.await
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to commit transaction"))?;

	// create jwt access token
	if let Some(user_session) = user_session {
		let jwt = AuthJwtPayload::from(user_session.clone());
		let token = jwt
			.serialize(global)
			.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to serialize jwt"))?;
		// create cookie
		let expiration = cookie::time::OffsetDateTime::from_unix_timestamp(user_session.expires_at.timestamp())
			.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to create expiration"))?;

		cookies.add(new_cookie(global, (AUTH_COOKIE, token.clone())).expires(expiration));

		Ok(format!(
			"{}?platform={}&token={}",
			global.config().api.connections.callback_url,
			platform,
			token
		))
	} else {
		Ok(format!(
			"{}?platform={}",
			global.config().api.connections.callback_url,
			platform,
		))
	}
}

pub fn handle_login(
	global: &Arc<Global>,
	session: Option<&UserSession>,
	platform: Platform,
	cookies: &Cookies,
) -> Result<String, RouteError<ApiError>> {
	// redirect to platform auth url
	let (url, scope, config) = match platform {
		Platform::Twitch => (TWITCH_AUTH_URL, TWITCH_AUTH_SCOPE, &global.config().api.connections.twitch),
		Platform::Discord => (DISCORD_AUTH_URL, DISCORD_AUTH_SCOPE, &global.config().api.connections.discord),
		Platform::Google => (GOOGLE_AUTH_URL, GOOGLE_AUTH_SCOPE, &global.config().api.connections.google),
		_ => {
			return Err((
				StatusCode::BAD_REQUEST,
				"unsupported platform",
				ApiError::ConnectionError(connections::ConnectionError::UnsupportedPlatform),
			)
				.into());
		}
	};

	let csrf = CsrfJwtPayload::new(session.map(|s| s.user_id));

	cookies.add(new_cookie(
		global,
		(
			CSRF_COOKIE,
			csrf.serialize(global)
				.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to serialize csrf"))?,
		),
	));

	let redirect_uri = format!("{}/v3/auth?callback=true&platform={}", global.config().api.base_url, platform);

	Ok(format!(
		"{}client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
		url,
		config.client_id,
		urlencoding::encode(&redirect_uri),
		urlencoding::encode(scope),
		csrf.random()
	))
}
