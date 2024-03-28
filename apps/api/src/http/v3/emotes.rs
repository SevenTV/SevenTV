use std::str::FromStr;
use std::sync::Arc;

use hyper::body::Incoming;
use hyper::header::CONTENT_TYPE;
use hyper::StatusCode;
use postgres_from_row::FromRow;
use scuffle_utils::http::ext::{OptionExt, ResultExt};
use scuffle_utils::http::router::Router;
use scuffle_utils::http::router::{builder::RouterBuilder, ext::RequestExt};
use scuffle_utils::http::RouteError;
use shared::http::Body;
use shared::object_id::ObjectId;
use shared::types::ImageHost;

use crate::database;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::RequestGlobalExt;

use super::types::{Emote, EmoteFlags, EmoteLifecycle, EmoteVersion, EmoteVersionState};

#[derive(utoipa::OpenApi)]
#[openapi(
	paths(create_emote, get_emote_by_id),
	components(schemas(Emote, EmoteVersion, EmoteVersionState, XEmoteData))
)]
pub struct Docs;

pub fn routes(_: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
	Router::builder().post("/", create_emote).get("/{id}", get_emote_by_id)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct XEmoteData {}

#[utoipa::path(
    post,
    path = "/v3/emotes",
    tag = "emotes",
    // Currently utoipa does not support multiple request body types so we use `image/*` as a placeholder
    // See https://github.com/juhaku/utoipa/pull/876
    request_body(content = &[u8], description = "Image Binary Data", content_type = "image/*"),
    responses(
        (status = 201, description = "Emote Created"),
    ),
    params(
        ("X-Emote-Data" = XEmoteData, Header, description = "The properties of the emote"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emotes/emotes.create.go#L58
pub async fn create_emote(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	todo!()
}

#[utoipa::path(
    get,
    path = "/v3/emotes/{id}",
    tag = "emotes",
    responses(
        (status = 200, description = "Emote", body = Emote),
        // (status = 404, description = "Emote Not Found", body = ApiError)
    ),
    params(
        ("id" = String, Path, description = "The ID of the emote"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emotes/emotes.by-id.go#L36
pub async fn get_emote_by_id(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	let global: Arc<Global> = req.get_global()?;
	let id = req.param("id").map_err_route((StatusCode::BAD_REQUEST, "missing id"))?;
	let id = ObjectId::from_str(id).map_ignore_err_route((StatusCode::BAD_REQUEST, "invalid id"))?;
	let rows = scuffle_utils::database::query(
		"SELECT * FROM emotes LEFT JOIN emote_files ON emotes.id = emote_files.emote_id WHERE emotes.id = $1",
	)
	.bind(id.into_ulid())
	.build()
	.fetch_all(&global.db())
	.await
	.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch emote"))?;

	let Some(first_row) = rows.first() else {
		return Err((StatusCode::NOT_FOUND, "emote not found").into());
	};
	let emote = database::Emote::from_row(first_row);
	let emote_files: Vec<_> = rows.into_iter().map(|r| database::EmoteFile::from_row(&r)).collect();
	let files = global
		.file_by_id_loader()
		.load_many(emote_files.iter().map(|f| f.file_id))
		.await
		.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to load files"))?;

	let host = ImageHost {
		url: format!("{}/emote/{}", global.config().api.cdn_base_url, ObjectId::from_ulid(emote.id)),
		files: emote_files
			.into_iter()
			.filter_map(|f| {
				let file = files.get(&f.file_id)?;
                Some(f.data.into_host_file(file.path.clone()))
			})
			.collect(),
	};

	let owner = match emote.owner_id {
		Some(owner) => {
			let user = global
				.user_by_id_loader()
				.load(&global, owner)
				.await
				.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to load user"))?;
			match user {
				Some(u) => Some(
					u.into_old_model(&global)
						.await
						.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to convert user"))?,
				),
				None => None,
			}
		}
		None => None,
	};

	let mut state = vec![];
	if emote.settings.listed {
		state.push(EmoteVersionState::Listed);
	}
	if emote.settings.personal {
		state.push(EmoteVersionState::Personal);
	} else {
		state.push(EmoteVersionState::NoPersonal);
	}

	let mut flags = EmoteFlags::empty();
	if emote.settings.zero_width {
		flags |= EmoteFlags::ZERO_WIDTH;
	}

	let emote = Emote {
		id: emote.id.into(),
		name: emote.default_name.clone(),
		flags,
		tags: emote.tags,
		lifecycle: EmoteLifecycle::LIVE,
		state: state.clone(),
		listed: emote.settings.listed,
		animated: emote.animated,
		owner,
		host: host.clone(),
		versions: vec![EmoteVersion {
			id: emote.id.into(),
			name: emote.default_name,
			description: String::new(),
			lifecycle: EmoteLifecycle::LIVE,
			state,
			listed: emote.settings.listed,
			animated: emote.animated,
			host: Some(host),
			created_at: emote.id.timestamp_ms(),
		}],
	};
	let data = serde_json::to_vec(&emote).map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to serialize emote"))?;
	let body = Body::Left(http_body_util::Full::new(data.into()));
	Ok(hyper::Response::builder()
		.status(StatusCode::OK)
		.header(CONTENT_TYPE, "application/json")
		.body(body)
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to build response"))?)
}
