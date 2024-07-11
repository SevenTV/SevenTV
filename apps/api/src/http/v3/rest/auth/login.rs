use std::sync::Arc;

use hyper::StatusCode;
use mongodb::bson::{doc, to_bson};
use shared::database::audit_log::{AuditLog, AuditLogData, AuditLogId, AuditLogUserData};
use shared::database::role::permissions::{PermissionsExt, UserPermission};
use shared::database::user::connection::{Platform, UserConnection, UserConnectionBuilder};
use shared::database::user::session::UserSession;
use shared::database::user::{User, UserBuilder, UserId};
use shared::database::{Collection, Id};
use shared::event_api::types::{ChangeFieldBuilder, ChangeFieldType, ChangeMapBuilder, EventType, ObjectKind};
use shared::old_types::{UserConnectionPartialModel, UserPartialModel};

use super::LoginRequest;
use crate::connections;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AUTH_COOKIE;
use crate::http::middleware::cookies::{new_cookie, Cookies};
use crate::http::v3::rest::types::UserConnectionModel;
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

	let platform = Platform::from(query.platform);

	// exchange code for access token
	let token = connections::exchange_code(
		global,
		platform,
		&code,
		format!(
			"{}/v3/auth?callback=true&platform={}",
			global.config().api.api_origin,
			query.platform
		),
	)
	.await?;

	// query user data from platform
	let user_data = connections::get_user_data(global, platform, &token.access_token).await?;

	let mut session = global.mongo().start_session().await.map_err(|err| {
		tracing::error!(error = %err, "failed to start session");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	session.start_transaction().await.map_err(|err| {
		tracing::error!(error = %err, "failed to start transaction");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	let mut user = User::collection(global.db())
		.find_one_and_update(
			doc! {
				"connections.platform": to_bson(&platform).expect("failed to convert to bson"),
				"connections.platform_id": &user_data.id,
			},
			doc! {
				"$set": {
					"connections.$.platform_username": &user_data.username,
					"connections.$.platform_display_name": &user_data.display_name,
					"connections.$.platform_avatar_url": &user_data.avatar,
					"connections.$.updated_at": chrono::Utc::now(),
				},
			},
		)
		.session(&mut session)
		.await
		.map_err(|err| {
			tracing::error!(error = %err, "failed to find user");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

	match (&user, csrf_payload.user_id) {
		(Some(user), Some(user_id)) => {
			if user.id != user_id {
				return Err(ApiError::new_const(
					StatusCode::BAD_REQUEST,
					"connection already paired with another user",
				));
			}
		}
		(Some(user), None) => {
			let connection = user
				.connections
				.iter()
				.find(|c| c.platform == platform && c.platform_id == user_data.id)
				.ok_or_else(|| {
					tracing::error!("connection not found");
					ApiError::INTERNAL_SERVER_ERROR
				})?;

			if !connection.allow_login {
				return Err(ApiError::new_const(
					StatusCode::UNAUTHORIZED,
					"connection is not allowed to login",
				));
			}
		}
		(None, None) => {
			// New user creation

			let new_connection = UserConnectionBuilder::default()
				.platform(platform)
				.platform_id(user_data.id.clone())
				.platform_username(user_data.username.clone())
				.platform_display_name(user_data.display_name.clone())
				.platform_avatar_url(user_data.avatar.clone())
				.build()
				.unwrap();
			user = Some(UserBuilder::default().connections(vec![new_connection]).build().unwrap());

			User::collection(global.db())
				.insert_one(user.as_ref().unwrap())
				.session(&mut session)
				.await
				.map_err(|err| {
					tracing::error!(error = %err, "failed to insert user");
					ApiError::INTERNAL_SERVER_ERROR
				})?;
		}
		_ => {}
	};

	let full_user = if let Some(user) = user {
		// This is correct for users that just got created aswell, as this will simply
		// load the default entitlements, as the user does not exist yet in the
		// database.
		global
			.user_loader()
			.load_user(user)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
	} else if let Some(user_id) = csrf_payload.user_id {
		global
			.user_loader()
			.load(global, user_id)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or_else(|| ApiError::new_const(StatusCode::BAD_REQUEST, "user not found"))?
	} else {
		unreachable!("user should be created or loaded");
	};

	if !full_user.has(UserPermission::Login) {
		return Err(ApiError::new_const(StatusCode::FORBIDDEN, "not allowed to login"));
	}

	let logged_connection = full_user
		.connections
		.iter()
		.find(|c| c.platform == platform && c.platform_id == user_data.id);

	if let Some(logged_connection) = logged_connection {
		if logged_connection.platform_avatar_url != user_data.avatar
			|| logged_connection.platform_username != user_data.username
			|| logged_connection.platform_display_name != user_data.display_name
		{
			// Update user connection
			if User::collection(global.db())
				.update_one(
					doc! {
						"_id": full_user.user.id,
						"connections.platform": to_bson(&platform).expect("failed to convert to bson"),
						"connections.platform_id": user_data.id,
					},
					doc! {
						"$set": {
							"connections.$.platform_username": &user_data.username,
							"connections.$.platform_display_name": &user_data.display_name,
							"connections.$.platform_avatar_url": &user_data.avatar,
							"connections.$.updated_at": chrono::Utc::now(),
							// This will trigger a search engine update
							"search_index.self_dirty": Id::<()>::new(),
						},
					},
				)
				.session(&mut session)
				.await
				.map_err(|err| {
					tracing::error!(error = %err, "failed to update user");
					ApiError::INTERNAL_SERVER_ERROR
				})?
				.matched_count == 0
			{
				tracing::error!("failed to update user, no matched count");
				return Err(ApiError::INTERNAL_SERVER_ERROR);
			}
		}
	} else {
		let new_connection = UserConnection {
			platform,
			platform_id: user_data.id,
			platform_username: user_data.username,
			platform_display_name: user_data.display_name,
			platform_avatar_url: user_data.avatar,
			allow_login: true,
			updated_at: chrono::Utc::now(),
			linked_at: chrono::Utc::now(),
		};

		if User::collection(global.db())
			.update_one(
				doc! {
					"_id": full_user.user.id,
				},
				doc! {
					"$push": {
						"connections": to_bson(&new_connection).expect("failed to convert to bson"),
					},
					"$set": {
						"search_index.self_dirty": Id::<()>::new(),
					},
				},
			)
			.session(&mut session)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to update user");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.modified_count
			> 0
		{
			let global_config = global
				.global_config_loader()
				.load(())
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

			global
				.event_api()
				.dispatch_event(
					EventType::UpdateUser,
					ChangeMapBuilder::default()
						.id(full_user.id.cast())
						.kind(ObjectKind::User)
						.actor(Some(UserPartialModel::from_db(
							full_user.clone(),
							&global_config,
							None,
							None,
							&global.config().api.cdn_origin,
						)))
						.pushed(vec![ChangeFieldBuilder::default()
							.key("connections")
							.ty(ChangeFieldType::Object)
							.nested(true)
							.index(full_user.connections.len())
							.value(
								serde_json::to_value(UserConnectionModel::from(UserConnectionPartialModel::from_db(
									new_connection,
									full_user.style.active_emote_set_id,
									global_config.normal_emote_set_slot_capacity,
								)))
								.map_err(|e| {
									tracing::error!(error = %e, "failed to serialize user connection");
									ApiError::INTERNAL_SERVER_ERROR
								})?,
							)
							.build()
							.unwrap()])
						.build()
						.unwrap(),
					full_user.id,
				)
				.await
				.map_err(|err| {
					tracing::error!(error = %err, "failed to dispatch event");
					ApiError::INTERNAL_SERVER_ERROR
				})?;
		} else {
			tracing::error!("failed to insert user connection, no modified count");
			return Err(ApiError::INTERNAL_SERVER_ERROR);
		}
	}

	let user_session = if csrf_payload.user_id.is_none() {
		let user_session = UserSession {
			id: Default::default(),
			user_id: full_user.id,
			// TODO maybe allow for this to be configurable
			expires_at: chrono::Utc::now() + chrono::Duration::days(30),
			last_used_at: chrono::Utc::now(),
		};

		UserSession::collection(global.db())
			.insert_one(&user_session)
			.session(&mut session)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to insert user session");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		Some(user_session)
	} else {
		None
	};

	AuditLog::collection(global.db())
		.insert_one(AuditLog {
			id: AuditLogId::new(),
			actor_id: Some(full_user.id),
			data: AuditLogData::User {
				target_id: full_user.id,
				data: AuditLogUserData::Login { platform },
			},
		})
		.session(&mut session)
		.await
		.map_err(|err| {
			tracing::error!(error = %err, "failed to insert audit log");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

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
		cookies.remove(&global, CSRF_COOKIE);

		format!(
			"{}/auth/callback?platform={}&token={}",
			global.config().api.website_origin,
			query.platform,
			token
		)
	} else {
		format!(
			"{}/auth/callback?platform={}",
			global.config().api.website_origin,
			query.platform
		)
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
			global.config().api.api_origin,
			platform
		)),
		urlencoding::encode(scope),
		csrf.random()
	);

	Ok(redirect_url)
}
