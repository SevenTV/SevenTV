use std::sync::Arc;

use axum::extract::{Json, State};
use axum::routing::post;
use axum::Router;
use shared::database::user::connection::Platform;
use shared::event_api::payload::{BridgeBody, Dispatch};
use shared::event_api::types::{ChangeMap, EventType, ObjectKind};
use shared::old_types::cosmetic::{CosmeticAvatarModel, CosmeticKind, CosmeticModel};
use shared::old_types::image::ImageHost;
use shared::old_types::UserPartialModel;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/event-api", post(event_api))
}

#[tracing::instrument(skip_all)]
async fn event_api(
	State(global): State<Arc<Global>>,
	Json(body): Json<BridgeBody>,
) -> Result<Json<Vec<Dispatch>>, ApiError> {
	let users = global
		.user_by_platform_username_loader
		.load_many(body.identifiers.iter().map(|i| (Platform::Twitch, i.username().to_owned())))
		.await
		.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load users"))?;

	let users = global
		.user_loader
		.load_fast_user_many(&global, users.into_values())
		.await
		.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load users"))?;

	let results = users
		.into_values()
		.filter_map(|user| {
			if let Some(pfp) = &user.active_profile_picture {
				Some(CosmeticModel {
					id: user.id,
					kind: CosmeticKind::Avatar,
					data: CosmeticAvatarModel {
						id: pfp.id,
						aas: "".to_owned(),
						host: ImageHost::from_image_set(&pfp.image_set, &global.config.api.cdn_origin),
						user: UserPartialModel::from_db(user, None, None, &global.config.api.cdn_origin),
					},
				})
			} else {
				None
			}
		})
		.map(|cosmetic| {
			let object = serde_json::to_value(&cosmetic).map_err(|e| {
				tracing::error!(error = %e, "failed to serialize cosmetic");
				ApiError::internal_server_error(ApiErrorCode::Unknown, "failed to serialize cosmetic")
			})?;

			Ok::<_, ApiError>(Dispatch {
				ty: EventType::CreateCosmetic,
				body: ChangeMap {
					id: cosmetic.data.user.id.cast(),
					kind: ObjectKind::Cosmetic,
					contextual: true,
					object,
					..Default::default()
				},
			})
		})
		.collect::<Result<Vec<_>, _>>()?;

	Ok(Json(results))
}
