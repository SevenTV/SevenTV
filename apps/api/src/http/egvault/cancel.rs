use std::ops::Deref;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use shared::database::product::subscription::{ProviderSubscriptionId, SubscriptionId, SubscriptionPeriod, SubscriptionState};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, UserPermission};
use shared::database::Id;
use shared::database::product::subscription::Subscription;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::extract::Path;
use crate::http::middleware::session::Session;
use crate::http::v3::rest::users::TargetUser;
use crate::paypal_api;
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

	let target = global
		.user_loader
		.load(&global, target_id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load target user"))?
		.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "target user not found"))?;

	if !target.has(UserPermission::Billing) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"this user isn't allowed to use billing features",
		));
	}

	if target_id != auth_user.id && !auth_user.has(UserPermission::ManageBilling) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"you are not allowed to manage billing",
		));
	}

	let req = RateLimitRequest::new(RateLimitResource::EgVaultPaymentMethod, &session);

	req.http(&global, async {
		let stripe_client = global.stripe_client.safe(Id::<()>::new()).await;

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
							stripe_client.client("update").await.deref(),
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
					}
					Some(ProviderSubscriptionId::Paypal(id)) => {
						let api_key = paypal_api::api_key(&global).await.map_err(TransactionError::Custom)?;

						// https://developer.paypal.com/docs/api/subscriptions/v1/#subscriptions_cancel
						let response = global
							.http_client
							.post(format!("https://api.paypal.com/v1/billing/subscriptions/{id}/cancel"))
							.bearer_auth(&api_key)
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

						if !response.status().is_success() {
							tracing::error!(status = %response.status(), "failed to cancel paypal subscription");
							return Err(TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::PaypalError,
								"failed to cancel paypal subscription",
							)));
						}
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
									search_updated_at: &None,
								},
							},
							None,
						)
						.await?;
					}
				}

				// This would get updated by the sub refresh job eventually but we want it to reflect instantly
				tx.update_one(filter::filter! {
					Subscription {
						#[query(rename = "_id", serde)]
						id: period.subscription_id,
					}
				}, update::update! {
					#[query(set)]
					Subscription {
						#[query(serde)]
						state: SubscriptionState::CancelAtEnd,
						updated_at: chrono::Utc::now(),
						search_updated_at: &None,
					}
				}, None).await.map_err(|e| {
					tracing::error!(error = %e, "failed to update subscription");
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::MutationError,
						"failed to update subscription",
					))
				})?;

				Ok(())
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

	let target_id = match target {
		TargetUser::Me => auth_user.id,
		TargetUser::Other(id) => id,
	};

	let target = global
		.user_loader
		.load(&global, target_id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load target user"))?
		.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "target user not found"))?;

	if !target.has(UserPermission::Billing) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"this user isn't allowed to use billing features",
		));
	}

	if target_id != auth_user.id && !auth_user.has(UserPermission::ManageBilling) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"you are not allowed to manage billing",
		));
	}

	let req = RateLimitRequest::new(RateLimitResource::EgVaultSubscribe, &session);

	req.http(&global, async {
		let stripe_client = global.stripe_client.safe(Id::<()>::new()).await;

		let res = with_transaction(&global, |mut tx| async move {
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
				.ok_or_else(|| {
					TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "subscription not found"))
				})?;

			match period.provider_id {
				Some(ProviderSubscriptionId::Stripe(id)) => {
					stripe::Subscription::update(
						stripe_client.client("update").await.deref(),
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

					// This would get updated by the sub refresh job eventually but we want it to reflect instantly
					tx.update_one(filter::filter! {
						Subscription {
							#[query(rename = "_id", serde)]
							id: period.subscription_id,
						}
					}, update::update! {
						#[query(set)]
						Subscription {
							#[query(serde)]
							state: SubscriptionState::Active,
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					}, None).await.map_err(|e| {
						tracing::error!(error = %e, "failed to update subscription");
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::MutationError,
							"failed to update subscription",
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
