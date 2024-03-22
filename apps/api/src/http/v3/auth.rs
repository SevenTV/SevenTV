use std::str::FromStr;
use std::sync::Arc;

use hyper::body::Incoming;
use hyper::StatusCode;
use scuffle_utils::database::deadpool_postgres::GenericClient;
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::RouteError;
use shared::http::Body;
use ulid::Ulid;

use crate::connections;
use crate::database::{UserConnection, UserConnectionPlatform};
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::{RequestGlobalExt, RequestQueryParamExt};

#[derive(utoipa::OpenApi)]
#[openapi(paths(root, logout, manual))]
pub struct Docs;

pub fn routes(_: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
	Router::builder()
		.get("/", root)
		.post("/logout", logout)
		.get("/manual", manual)
}

#[utoipa::path(
    get,
    path = "/v3/auth",
    tag = "auth",
    responses(
        (status = 307, description = "Auth Redirect"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/auth.route.go#L47
async fn root(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	let global: Arc<Global> = req.get_global()?;
	let platform = req
		.query_param("platform")
		.and_then(|p| UserConnectionPlatform::from_str(&p).ok())
		.ok_or((StatusCode::BAD_REQUEST, "unsupported account provider"))?;
	let callback = req.query_param("callback").is_some_and(|c| c == "true");
	if callback {
		let state = req.query_param("state").ok_or((StatusCode::BAD_REQUEST, "missing state"))?;
		// TODO: validate csrf
		let code = req.query_param("code").ok_or((StatusCode::BAD_REQUEST, "missing code"))?;
		// exchange code for access token
		let token = connections::exchange_code(&global, platform, &code)
			.await
			.map_err(ApiError::from)?;
		// query user data from platform
		let user_data = connections::get_user_data(&global, platform, &token.access_token)
			.await
			.map_err(ApiError::from)?;
		let connection: Option<UserConnection> =
			scuffle_utils::database::query("SELECT * FROM user_connections WHERE platform = $1 AND platform_id = $2")
				.bind(platform)
				.bind(user_data.id.clone())
				.build_query_as()
				.fetch_optional(global.db())
				.await
				.map_err(ApiError::from)?;
		if connection.is_some() {
			// update user connection
			scuffle_utils::database::query("UPDATE user_connections SET platform_username = $1, platform_display_name = $2, platform_avatar_url = $3, updated_at = NOW() WHERE platform = $4 AND platform_id = $5")
				.bind(user_data.username)
				.bind(user_data.display_name)
				.bind(user_data.avatar)
				.bind(platform)
				.bind(user_data.id)
				.build()
				.execute(global.db())
				.await
				.map_err(ApiError::from)?;
		} else {
			let mut client = global.db().get().await.map_err(ApiError::from)?;
			let tx = client.transaction().await.map_err(ApiError::from)?;
			// create user
			let user_id = Ulid::new();
			scuffle_utils::database::query("INSERT INTO users (id) VALUES ($1)")
				.bind(user_id)
				.build()
				.execute(&tx)
				.await
				.map_err(ApiError::from)?;
			// create user connection
			scuffle_utils::database::query("INSERT INTO user_connections (id, user_id, platform, platform_id, platform_username, platform_display_name, platform_avatar_url) VALUES ($1, $2, $3, $4, $5, $6, $7)")
				.bind(Ulid::new())
				.bind(user_id)
				.bind(platform)
				.bind(user_data.id)
				.bind(user_data.username)
				.bind(user_data.display_name)
				.bind(user_data.avatar)
				.build()
				.execute(&tx)
				.await
				.map_err(ApiError::from)?;
			tx.commit().await.map_err(ApiError::from)?;
		}
		// create session
		// create jwt access token
		// set cookie and redirect to website
		todo!()
	} else {
		todo!()
	}
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
	todo!()
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
	todo!()
}
