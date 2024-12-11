use std::sync::Arc;

use axum::extract::State;
use axum::routing::post;
use axum::{Extension, Json, Router};
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy};
use shared::database::product::special_event::SpecialEventId;
use shared::database::product::subscription::SubscriptionId;
use shared::database::queries::filter;
use shared::database::role::permissions::{AdminPermission, PermissionsExt};
use shared::database::user::connection::Platform;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::transactions::{transaction_with_mutex, GeneralMutexKey, TransactionError};

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/create", post(create_event))
}

#[derive(Debug, Clone, serde::Deserialize)]
struct CreateEventRequest {
	twitch_id: String,
	special_event_id: SpecialEventId,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CreateEventResponse {
	success: bool,
}

async fn create_event(
	State(global): State<Arc<Global>>,
	Extension(session): Extension<Session>,
	Json(create_event): Json<CreateEventRequest>,
) -> Result<Json<CreateEventResponse>, ApiError> {
	let Some(user_session) = session.user_session() else {
		return Err(ApiError::unauthorized(ApiErrorCode::LackingPrivileges, "user not logged in"));
	};

	let has_permission = user_session
		.extensions
		.get_array("events_create")
		.map(|arr| arr.contains(&create_event.special_event_id.into()))
		.unwrap_or(false);

	if !session.has(AdminPermission::Admin) || !has_permission {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			format!(
				"you do not have permission to create an event for {}",
				create_event.special_event_id
			),
		));
	}

	let event = global
		.special_event_by_id_loader
		.load(create_event.special_event_id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load special event"))?
		.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "special event not found"))?;

	let user = global
		.user_by_platform_id_loader
		.load((Platform::Twitch, create_event.twitch_id))
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
		.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

	let products = global
		.subscription_products_loader
		.load(())
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription"))?
		.ok_or_else(|| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription"))?;

	// We only have 1 for now.
	let product = products
		.into_iter()
		.next()
		.ok_or_else(|| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription"))?;

	let from = EntitlementEdgeKind::Subscription {
		subscription_id: SubscriptionId {
			product_id: product.id,
			user_id: user.id,
		},
	};

	transaction_with_mutex(&global, Some(GeneralMutexKey::User(user.id).into()), |mut tx| async move {
		tx.delete(
			filter::filter! {
				EntitlementEdge {
					#[query(flatten)]
					id: EntitlementEdgeId {
						#[query(serde)]
						from: &from,
						#[query(serde)]
						managed_by: Some(EntitlementEdgeManagedBy::SpecialEvent {
							special_event_id: event.id,
						}),
					},
				}
			},
			None,
		)
		.await?;

		tx.insert_one(
			EntitlementEdge {
				id: EntitlementEdgeId {
					from,
					to: EntitlementEdgeKind::SpecialEvent {
						special_event_id: create_event.special_event_id,
					},
					managed_by: Some(EntitlementEdgeManagedBy::SpecialEvent {
						special_event_id: event.id,
					}),
				},
			},
			None,
		)
		.await?;

		Ok::<_, TransactionError<ApiError>>(())
	})
	.await
	.map_err(|err| {
		tracing::error!("failed to create event: {}", err);
		ApiError::internal_server_error(ApiErrorCode::TransactionError, "failed to create event")
	})?;

	Ok(Json(CreateEventResponse { success: true }))
}
