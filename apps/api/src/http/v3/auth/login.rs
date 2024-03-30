use std::sync::Arc;

use cookie::CookieJar;
use hyper::body::Incoming;
use hyper::StatusCode;
use scuffle_utils::http::ext::{OptionExt, ResultExt};
use scuffle_utils::http::RouteError;
use ulid::Ulid;

use crate::connections;
use crate::database::{UserConnection, UserConnectionPlatform, UserSession};
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::{new_cookie, AUTH_COOKIE};
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
	platform: UserConnectionPlatform,
	req: &hyper::Request<Incoming>,
	cookies: &mut CookieJar,
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

	CsrfJwtPayload::verify(global, csrf_cookie.value())
		.and_then(|p| p.validate_random(&state))
		.map(|valid| {
			if valid {
				Ok(())
			} else {
				Err((StatusCode::BAD_REQUEST, "invalid state"))
			}
		})
		.map_err_route((StatusCode::BAD_REQUEST, "invalid csrf"))??;

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

	let connection: Option<UserConnection> =
		scuffle_utils::database::query("SELECT * FROM user_connections WHERE platform = $1 AND platform_id = $2")
			.bind(platform)
			.bind(user_data.id.clone())
			.build_query_as()
			.fetch_optional(global.db())
			.await
			.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to query connection"))?;

	let mut client = global
		.db()
		.get()
		.await
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to initialize db client"))?;

	let tx = client
		.transaction()
		.await
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to initialize db transaction"))?;

	let user_id = if let Some(connection) = connection {
		if !connection.allow_login {
			return Err((
				StatusCode::FORBIDDEN,
				"connection is not allowed to login",
				ApiError::ConnectionError(connections::ConnectionError::LoginNotAllowed),
			)
				.into());
		}

		// update user connection
		scuffle_utils::database::query("UPDATE user_connections SET platform_username = $4, platform_display_name = $5, platform_avatar_url = $6, updated_at = NOW() WHERE platform = $7 AND platform_id = $8")
            .bind(user_data.username)
            .bind(user_data.display_name)
            .bind(user_data.avatar)
            .bind(platform)
            .bind(user_data.id)
            .build()
            .execute(&tx)
            .await
            .map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to update user connection"))?;

		connection.user_id
	} else {
		let user_id = if let Some(auth) = req.extensions().get::<UserSession>() {
			auth.user_id
		} else {
			// create user
			let user_id = Ulid::new();

			scuffle_utils::database::query("INSERT INTO users (id) VALUES ($1)")
				.bind(user_id)
				.build()
				.execute(&tx)
				.await
				.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to create user"))?;

			user_id
		};

		// create user connection
		scuffle_utils::database::query("INSERT INTO user_connections (id, user_id, main_connection, platform, platform_id, platform_username, platform_display_name, platform_avatar_url) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)")
            .bind(Ulid::new())
            .bind(user_id)
            .bind(true)
            .bind(platform)
            .bind(user_data.id)
            .bind(user_data.username)
            .bind(user_data.display_name)
            .bind(user_data.avatar)
            .build()
            .execute(&tx)
            .await
            .map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to create user connection"))?;

		user_id
	};
	// create session
	let session: UserSession = scuffle_utils::database::query(
		"INSERT INTO user_sessions (id, user_id, expires_at) VALUES ($1, $2, $3) RETURNING *",
	)
	.bind(Ulid::new())
	.bind(user_id)
	.bind(chrono::Utc::now() + chrono::Duration::days(30))
	.build_query_as()
	.fetch_one(&tx)
	.await
	.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to create user connection"))?;

	// create jwt access token
	let jwt = AuthJwtPayload::from(session.clone());
	let token = jwt
		.serialize(global)
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to serialize jwt"))?;
	// create cookie
	let expiration = cookie::time::OffsetDateTime::from_unix_timestamp(session.expires_at.naive_utc().timestamp())
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to create expiration"))?;

	cookies.add(new_cookie(global, (AUTH_COOKIE, token.clone())).expires(expiration));

	tx.commit()
		.await
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to commit transcation"))?;

	Ok(format!(
		"{}?platform={}&token={}",
		global.config().api.connections.callback_url,
		platform,
		token
	))
}

pub fn handle_login(
	global: &Arc<Global>,
	platform: UserConnectionPlatform,
	cookies: &mut CookieJar,
) -> Result<String, RouteError<ApiError>> {
	// redirect to platform auth url
	let (url, scope, config) = match platform {
		UserConnectionPlatform::Twitch => (TWITCH_AUTH_URL, TWITCH_AUTH_SCOPE, &global.config().api.connections.twitch),
		UserConnectionPlatform::Discord => (DISCORD_AUTH_URL, DISCORD_AUTH_SCOPE, &global.config().api.connections.discord),
		UserConnectionPlatform::Google => (GOOGLE_AUTH_URL, GOOGLE_AUTH_SCOPE, &global.config().api.connections.google),
		_ => {
			return Err((
				StatusCode::BAD_REQUEST,
				"unsupported platform",
				ApiError::ConnectionError(connections::ConnectionError::UnsupportedPlatform),
			)
				.into());
		}
	};

	let csrf = CsrfJwtPayload::new();

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
