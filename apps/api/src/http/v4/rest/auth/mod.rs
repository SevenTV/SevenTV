use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::{Redirect, Response};
use axum::routing::get;
use axum::{Extension, Router};
use shared::database::role::permissions::RateLimitResource;
use shared::database::user::connection::Platform;
use shared::database::Id;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::cookies::{new_cookie, Cookies};
use crate::http::middleware::session::Session;
use crate::ratelimit::RateLimitRequest;

const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize?";
const TWITCH_AUTH_SCOPE: &str = "";

const DISCORD_AUTH_URL: &str = "https://discord.com/oauth2/authorize?";
const DISCORD_AUTH_SCOPE: &str = "identify";

const GOOGLE_AUTH_URL: &str =
	"https://accounts.google.com/o/oauth2/v2/auth?access_type=offline&include_granted_scopes=true&";
const GOOGLE_AUTH_SCOPE: &str = "https://www.googleapis.com/auth/youtube.readonly";

const CSRF_COOKIE: &str = "seventv-login-csrf";

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/login", get(login))
		.route("/logout", get(logout))
		.route("/callback", get(callback))
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoginRequestPlatform {
	Twitch,
	Discord,
	Google,
	Kick,
}

impl LoginRequestPlatform {
	pub fn as_str(&self) -> &'static str {
		match self {
			LoginRequestPlatform::Twitch => "twitch",
			LoginRequestPlatform::Discord => "discord",
			LoginRequestPlatform::Google => "google",
			LoginRequestPlatform::Kick => "kick",
		}
	}
}

impl From<LoginRequestPlatform> for Platform {
	fn from(platform: LoginRequestPlatform) -> Self {
		match platform {
			LoginRequestPlatform::Twitch => Platform::Twitch,
			LoginRequestPlatform::Discord => Platform::Discord,
			LoginRequestPlatform::Google => Platform::Google,
			LoginRequestPlatform::Kick => Platform::Kick,
		}
	}
}

#[derive(Debug, serde::Deserialize)]
struct LoginRequest {
	pub platform: LoginRequestPlatform,
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

		let redirect_uri = global
			.config
			.api
			.api_origin
			.join(&format!("/v4/auth/callback&platform={}", query.platform.as_str()))
			.map_err(|e| {
				tracing::error!(err = %e, "failed to generate redirect_uri");
				ApiError::internal_server_error(ApiErrorCode::Unknown, "failed to generate redirect_uri")
			})?;

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

async fn callback() -> &'static str {
	"test"
}

async fn logout() -> &'static str {
	"test"
}
