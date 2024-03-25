use std::str::FromStr;
use std::sync::Arc;

use cookie::{Cookie, CookieJar};
use hyper::body::Incoming;
use hyper::header::ToStrError;
use hyper::{Response, StatusCode};
use scuffle_utils::database::deadpool_postgres::GenericClient;
use scuffle_utils::http::ext::{OptionExt, ResultExt};
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::RouteError;
use shared::http::Body;
use ulid::Ulid;

use crate::connections;
use crate::database::{UserConnection, UserConnectionPlatform, UserSession};
use crate::global::Global;
use crate::http::cookies::new_cookie;
use crate::http::error::ApiError;
use crate::http::{RequestGlobalExt, RequestQueryParamExt};
use crate::jwt::{AuthJwtPayload, CsrfJwtPayload, JwtState};

#[derive(utoipa::OpenApi)]
#[openapi(paths(root, logout, manual))]
pub struct Docs;

pub fn routes(_: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
	Router::builder()
		.get("/", root)
		.post("/logout", logout)
		.get("/manual", manual)
}

const CSRF_COOKIE: &str = "seventv-csrf";
const AUTH_COOKIE: &str = "seventv-auth";

#[utoipa::path(
    get,
    path = "/v3/auth",
    tag = "auth",
    responses(
        (status = 303, description = "Auth Redirect"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/auth.route.go#L47
async fn root(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	let global: Arc<Global> = req.get_global()?;
	let platform = req
		.query_param("platform")
		.and_then(|p| UserConnectionPlatform::from_str(&p).ok())
		.map_err_route((StatusCode::BAD_REQUEST, "invalid account provider"))?;

	let mut jar = req
		.headers()
		.get_all(hyper::header::COOKIE)
		.iter()
		.try_fold(CookieJar::new(), |mut jar, h| {
			for c in Cookie::split_parse_encoded(h.to_str()?) {
				match c {
					Ok(cookie) => jar.add_original(cookie.into_owned()),
					Err(e) => tracing::debug!("failed to parse a cookie {}", e),
				}
			}
			Ok::<CookieJar, ToStrError>(jar)
		})
		.map_err_route((StatusCode::BAD_REQUEST, "invalid cookie header"))?;

	let callback = req.query_param("callback").is_some_and(|c| c == "true");
	let mut response = Response::builder();
	if callback {
		let auth = jar
			.get(AUTH_COOKIE)
			.map(|c| AuthJwtPayload::verify(&global, c.value()))
			.flatten();

		// validate csrf
		let csrf_cookie = jar
			.get(CSRF_COOKIE)
			.map_err_route((StatusCode::BAD_REQUEST, "missing csrf cookie"))?;
		let state = req
			.query_param("state")
			.map_err_route((StatusCode::BAD_REQUEST, "missing state"))?;
		let csrf = CsrfJwtPayload::verify(&global, csrf_cookie.value())
			.and_then(|p| p.validate_random(&state))
			.map_err_route((StatusCode::BAD_REQUEST, "invalid csrf"))?;
		if !csrf {
			return Err((StatusCode::BAD_REQUEST, "invalid state").into());
		}

		let code = req
			.query_param("code")
			.map_err_route((StatusCode::BAD_REQUEST, "missing code"))?;
		// exchange code for access token
		let token = connections::exchange_code(
			&global,
			&platform,
			&code,
			format!(
				"{}/v3/auth?callback=true&platform={}",
				global.config().api.base_url,
				platform.to_string()
			),
		)
		.await
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to exchange code"))?;
		// query user data from platform
		let user_data = connections::get_user_data(&global, platform, &token.access_token)
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
		// expires in with a safety margin of 1 minute
		let expires_at = chrono::Utc::now() + chrono::Duration::seconds(token.expires_in) - chrono::Duration::minutes(1);

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
			scuffle_utils::database::query("UPDATE user_connections SET platform_access_token = $1, platform_access_token_expires_at = $2, platform_refresh_token = COALESCE($3, platform_refresh_token), platform_username = $4, platform_display_name = $5, platform_avatar_url = $6, updated_at = NOW() WHERE platform = $7 AND platform_id = $8")
				.bind(token.access_token)
				.bind(expires_at)
				.bind(token.refresh_token)
				.bind(user_data.username)
				.bind(user_data.display_name)
				.bind(user_data.avatar)
				.bind(platform)
				.bind(user_data.id)
				.build()
				.execute(global.db())
				.await
				.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to update user connection"))?;
			connection.user_id
		} else {
			let user_id = if let Some(auth) = auth {
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
			scuffle_utils::database::query("INSERT INTO user_connections (id, user_id, platform, platform_access_token, platform_access_token_expires_at, platform_refresh_token, platform_id, platform_username, platform_display_name, platform_avatar_url) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
				.bind(Ulid::new())
				.bind(user_id)
				.bind(platform)
				.bind(token.access_token)
				.bind(expires_at)
				.bind(token.refresh_token)
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
			.serialize(&global)
			.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to serialize jwt"))?;
		// create cookie
		let expiration = cookie::time::OffsetDateTime::from_unix_timestamp(session.expires_at.naive_utc().timestamp())
			.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to create expiration"))?;
		jar.add(new_cookie(&global, (AUTH_COOKIE, token.clone())).expires(expiration));
		tx.commit()
			.await
			.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to commit transcation"))?;
		// redirect to callback url
		response = response.header(
			hyper::header::LOCATION,
			format!(
				"{}?platform={}&token={}",
				global.config().api.connections.callback_url,
				platform.to_string(),
				token
			),
		);
	} else {
		// redirect to platform auth url
		let (url, scope, config) = match platform {
			UserConnectionPlatform::Twitch => (
				"https://id.twitch.tv/oauth2/authorize?",
				"",
				&global.config().api.connections.twitch,
			),
			UserConnectionPlatform::Discord => (
				"https://discord.com/oauth2/authorize?",
				"identify",
				&global.config().api.connections.discord,
			),
			UserConnectionPlatform::Google => (
				"https://accounts.google.com/o/oauth2/v2/auth?access_type=offline&include_granted_scopes=true&",
				"https://www.googleapis.com/auth/youtube.readonly",
				&global.config().api.connections.google,
			),
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
		jar.add(new_cookie(
			&global,
			(
				CSRF_COOKIE,
				csrf.serialize(&global)
					.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to serialize csrf"))?,
			),
		));
		let redirect_uri = format!(
			"{}/v3/auth?callback=true&platform={}",
			global.config().api.base_url,
			platform.to_string()
		);
		response = response.header(
			hyper::header::LOCATION,
			format!(
				"{}client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
				url,
				config.client_id,
				urlencoding::encode(&redirect_uri),
				urlencoding::encode(&scope),
				csrf.random()
			),
		);
	}

	for cookie in jar.delta() {
		response = response.header(hyper::header::SET_COOKIE, cookie.to_string());
	}
	Ok(response
		.status(StatusCode::SEE_OTHER)
		// empty body
		.body(http_body_util::Either::Left(http_body_util::Full::new(
			hyper::body::Bytes::new(),
		)))
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to build response"))?)
}

#[utoipa::path(
    post,
    path = "/v3/auth/logout",
    tag = "auth",
    responses(
        (status = 204, description = "Logout"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/logout.auth.route.go#L29
async fn logout(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	let global: Arc<Global> = req.get_global()?;
	let mut jar = req
		.headers()
		.get_all(hyper::header::COOKIE)
		.iter()
		.try_fold(CookieJar::new(), |mut jar, h| {
			for c in Cookie::split_parse_encoded(h.to_str()?) {
				match c {
					Ok(cookie) => jar.add_original(cookie.into_owned()),
					Err(e) => tracing::debug!("failed to parse a cookie {}", e),
				}
			}
			Ok::<CookieJar, ToStrError>(jar)
		})
		.map_err_route((StatusCode::BAD_REQUEST, "invalid cookie header"))?;

	if let Some(cookie) = jar.get(AUTH_COOKIE) {
		let auth = AuthJwtPayload::verify(&global, cookie.value())
			.map_err_route((StatusCode::UNAUTHORIZED, "invalid auth token"))?;
		scuffle_utils::database::query("DELETE FROM user_sessions WHERE id = $1")
			.bind(auth.session_id)
			.build()
			.execute(global.db())
			.await
			.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to delete session"))?;
		jar.remove(Cookie::build(AUTH_COOKIE));
	}

	let mut response = Response::builder();
	for cookie in jar.delta() {
		response = response.header(hyper::header::SET_COOKIE, cookie.to_string());
	}
	Ok(response
		.status(StatusCode::NO_CONTENT)
		// empty body
		.body(http_body_util::Either::Left(http_body_util::Full::new(
			hyper::body::Bytes::new(),
		)))
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to build response"))?)
}

#[utoipa::path(
    get,
    path = "/v3/auth/manual",
    tag = "auth",
    responses(
        (status = 200, description = "Manual Auth"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/manual.route.go#L41
async fn manual(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	unimplemented!("kick auth is not implemented yet")
}
