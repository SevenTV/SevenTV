use std::ops::Deref;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use shared::database::product::subscription::{ProviderSubscriptionId, SubscriptionId, SubscriptionPeriod};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, UserPermission};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::extract::Path;
use crate::http::middleware::session::Session;
use crate::http::v3::rest::users::TargetUser;
use crate::ratelimit::RateLimitRequest;
use crate::transactions::{with_transaction, TransactionError};

pub async fn cancel_subscription(
	State(global): State<Arc<Global>>,
	Path(target): Path<TargetUser>,
	Extension(session): Extension<Session>,
) -> Result<impl IntoResponse, ApiError> {
	let auth_user = session.user()?;

	let target_id = match target {
		TargetUser::Me => auth_user.id,
		TargetUser::Other(id) => id,
	};

	// TODO: is this the right permission?
	if !auth_user.has(UserPermission::ManageAny) && target_id != auth_user.id {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"you are not allowed to manage this user",
		));
	}

	let req = RateLimitRequest::new(RateLimitResource::EgVaultPaymentMethod, &session);

	req.http(&global, async {
		let stripe_client = global.stripe_client.safe().await;

		let res = with_transaction(&global, |mut tx| {
			let global = Arc::clone(&global);

			async move {
				let period = tx
					.find_one(
						filter::filter! {
							SubscriptionPeriod {
								#[query(flatten)]
								subscription_id: SubscriptionId {
									user_id: target_id,
								},
								#[query(selector = "lt")]
								start: chrono::Utc::now(),
								#[query(selector = "gt")]
								end: chrono::Utc::now(),
							}
						},
						None,
					)
					.await?
					.ok_or(TransactionError::Custom(ApiError::not_found(
						ApiErrorCode::BadRequest,
						"subscription not found",
					)))?;

				match period.provider_id {
					Some(ProviderSubscriptionId::Stripe(id)) => {
						stripe::Subscription::update(
							stripe_client.client(()).await.deref(),
							&id,
							stripe::UpdateSubscription {
								cancel_at_period_end: Some(true),
								..Default::default()
							},
						)
						.await
						.map_err(|e| {
							tracing::error!(error = %e, "failed to update stripe subscription");
							TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::StripeError,
								"failed to update stripe subscription",
							))
						})?;

						Ok(())
					}
					Some(ProviderSubscriptionId::Paypal(id)) => {
						// https://developer.paypal.com/docs/api/subscriptions/v1/#subscriptions_cancel
						global
							.http_client
							.post(format!("https://api.paypal.com/v1/billing/subscriptions/{id}/cancel"))
							.bearer_auth(&global.config.api.paypal.api_key)
							.json(&serde_json::json!({
								"reason": "Subscription canceled by user"
							}))
							.send()
							.await
							.map_err(|e| {
								tracing::error!(error = %e, "failed to cancel paypal subscription");
								TransactionError::Custom(ApiError::internal_server_error(
									ApiErrorCode::PaypalError,
									"failed to cancel paypal subscription",
								))
							})?;

						Ok(())
					}
					None => {
						// This is a gifted or system subscription
						// End the current period right away

						tx.update_one(
							filter::filter! {
								SubscriptionPeriod {
									#[query(rename = "_id")]
									id: period.id,
								}
							},
							update::update! {
								#[query(set)]
								SubscriptionPeriod {
									end: chrono::Utc::now(),
									updated_at: chrono::Utc::now(),
								},
							},
							None,
						)
						.await?;

						Ok(())
					}
				}
			}
		})
		.await;

		match res {
			Ok(_) => Ok(StatusCode::OK),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	})
	.await
}

pub async fn reactivate_subscription(
	State(global): State<Arc<Global>>,
	Path(target): Path<TargetUser>,
	Extension(session): Extension<Session>,
) -> Result<impl IntoResponse, ApiError> {
	let auth_user = session.user()?;

	let target_user_id = match target {
		TargetUser::Me => auth_user.id,
		TargetUser::Other(id) => id,
	};

	// TODO: is this the right permission?
	if !auth_user.has(UserPermission::ManageAny) && target_user_id != auth_user.id {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"you are not allowed to manage this user",
		));
	}

	let req = RateLimitRequest::new(RateLimitResource::EgVaultSubscribe, &session);

	req.http(&global, async {
		let stripe_client = global.stripe_client.safe().await;

		let res = with_transaction(&global, |mut tx| async move {
			let period = tx
				.find_one(
					filter::filter! {
						SubscriptionPeriod {
							#[query(flatten)]
							subscription_id: SubscriptionId {
								user_id: target_user_id,
							},
							#[query(selector = "lt")]
							start: chrono::Utc::now(),
							#[query(selector = "gt")]
							end: chrono::Utc::now(),
						}
					},
					None,
				)
				.await?
				.ok_or_else(|| {
					TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "subscription not found"))
				})?;

			match period.provider_id {
				Some(ProviderSubscriptionId::Stripe(id)) => {
					stripe::Subscription::update(
						stripe_client.client(()).await.deref(),
						&id,
						stripe::UpdateSubscription {
							cancel_at_period_end: Some(false),
							..Default::default()
						},
					)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to update stripe subscription");
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::MutationError,
							"failed to update stripe subscription",
						))
					})?;

					Ok(())
				}
				_ => Err(TransactionError::Custom(ApiError::not_implemented(
					ApiErrorCode::BadRequest,
					"this subscription cannot be reactivated",
				))),
			}
		})
		.await;

		match res {
			Ok(_) => Ok(StatusCode::OK),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	})
	.await
}
