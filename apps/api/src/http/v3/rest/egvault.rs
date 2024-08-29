use std::sync::Arc;

use axum::{
	extract::State,
	http::StatusCode,
	routing::{get, patch, post},
	Extension, Json, Router,
};
use bson::doc;
use futures::TryStreamExt;
use shared::database::{
	entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy},
	product::{
		codes::{CodeEffect, RedeemCode},
		subscription::{ProviderSubscriptionId, SubscriptionPeriod, SubscriptionPeriodId},
		SubscriptionBenefitCondition, TimePeriod,
	},
	queries::update,
	role::permissions::PermissionsExt,
	MongoCollection,
};
use shared::database::{product::SubscriptionProduct, queries::filter};

use crate::{
	global::Global,
	http::{
		error::ApiError,
		extract::{Path, Query},
		middleware::auth::AuthSession,
	},
	transactions::{with_transaction, TransactionError},
};

use super::{types, users::TargetUser};

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/subscriptions", post(subscribe))
		.route("/subscriptions/:target", get(subscription).delete(cancel_subscription))
		.route("/subscription/:target/reactivate", post(reactivate_subscription))
		.route("/subscription/:target/payment-method", patch(payment_method))
		.route("/products", get(products))
		.route("/redeem", post(redeem))
}

#[derive(Debug, serde::Deserialize)]
struct SubscribeQuery {
	renew_interval: SubscriptionRenewInterval,
	/// only "stripe" allowed
	payment_method: String,
	/// always true
	next: bool,
	gift_for: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SubscribeBody {
	prefill: Prefill,
}

#[derive(Debug, serde::Deserialize)]
struct Prefill {
	first_name: String,
	last_name: String,
	email: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum SubscriptionRenewInterval {
	Monthly,
	Yearly,
}

#[derive(Debug, serde::Serialize)]
struct SubscribeResponse {
	/// Url that the website will open in a new tab
	url: String,
	/// The user id of the user that receives the subscription
	user_id: String,
}

async fn subscribe(
	State(global): State<Arc<Global>>,
	Query(query): Query<SubscribeQuery>,
	auth_session: Option<Extension<AuthSession>>,
	Json(body): Json<SubscribeBody>,
) -> Result<Json<SubscribeResponse>, ApiError> {
	let auth_session = auth_session.ok_or(ApiError::UNAUTHORIZED)?;

	if query.payment_method != "stripe" {
		return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "payment method not supported"));
	}
}

async fn subscription(
	State(global): State<Arc<Global>>,
	Path(target): Path<TargetUser>,
	auth_session: Option<Extension<AuthSession>>,
) -> Result<Json<types::Subscription>, ApiError> {
	let user = match target {
		TargetUser::Me => auth_session.ok_or(ApiError::UNAUTHORIZED)?.user_id(),
		TargetUser::Other(id) => id,
	};

	todo!()
}

async fn cancel_subscription(
	State(global): State<Arc<Global>>,
	Path(target): Path<TargetUser>,
	auth_session: Option<Extension<AuthSession>>,
) -> Result<StatusCode, ApiError> {
	let auth_session = auth_session.ok_or(ApiError::UNAUTHORIZED)?;

	let user = match target {
		TargetUser::Me => auth_session.user_id(),
		TargetUser::Other(id) => id,
	};

	if user != auth_session.user_id() {
		// TODO: allow with certain permissions
		return Err(ApiError::FORBIDDEN);
	}

	let res = with_transaction(&global, |mut tx| {
		let global = Arc::clone(&global);

		async move {
			let period = tx
				.find_one(
					filter::filter! {
						SubscriptionPeriod {
							user_id: user,
							#[query(selector = "lt")]
							start: chrono::Utc::now(),
							#[query(selector = "gt")]
							end: chrono::Utc::now(),
						}
					},
					None,
				)
				.await?
				.ok_or(TransactionError::custom(ApiError::NOT_FOUND))?;

			match period.subscription_id {
				ProviderSubscriptionId::Stripe(id) => {
					stripe::Subscription::update(
						&global.stripe_client,
						&id,
						stripe::UpdateSubscription {
							cancel_at_period_end: Some(true),
							..Default::default()
						},
					)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to update stripe subscription");
						TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
					})?;
				}
				ProviderSubscriptionId::Paypal(id) => {
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
							TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
						})?;
				}
			}

			Ok(())
		}
	})
	.await;

	match res {
		Ok(_) => Ok(StatusCode::OK),
		Err(TransactionError::Custom(e)) => Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			Err(ApiError::INTERNAL_SERVER_ERROR)
		}
	}
}

async fn reactivate_subscription(
	State(global): State<Arc<Global>>,
	Path(target): Path<TargetUser>,
	auth_session: Option<Extension<AuthSession>>,
) -> Result<StatusCode, ApiError> {
	let auth_session = auth_session.ok_or(ApiError::UNAUTHORIZED)?;

	let user = match target {
		TargetUser::Me => auth_session.user_id(),
		TargetUser::Other(id) => id,
	};

	if user != auth_session.user_id() {
		// TODO: allow with certain permissions
		return Err(ApiError::FORBIDDEN);
	}

	let res = with_transaction(&global, |mut tx| {
		let global = Arc::clone(&global);

		async move {
			let period = tx
				.find_one(
					filter::filter! {
						SubscriptionPeriod {
							user_id: user,
							#[query(selector = "lt")]
							start: chrono::Utc::now(),
							#[query(selector = "gt")]
							end: chrono::Utc::now(),
						}
					},
					None,
				)
				.await?
				.ok_or(TransactionError::custom(ApiError::NOT_FOUND))?;

			match period.subscription_id {
				ProviderSubscriptionId::Stripe(id) => {
					stripe::Subscription::update(
						&global.stripe_client,
						&id,
						stripe::UpdateSubscription {
							cancel_at_period_end: Some(false),
							..Default::default()
						},
					)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to update stripe subscription");
						TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
					})?;

					Ok(())
				}
				ProviderSubscriptionId::Paypal(_) => Err(TransactionError::custom(ApiError::new_const(
					StatusCode::NOT_IMPLEMENTED,
					"Paypal subscriptions cannot be reactivated",
				))),
			}
		}
	})
	.await;

	match res {
		Ok(_) => Ok(StatusCode::OK),
		Err(TransactionError::Custom(e)) => Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			Err(ApiError::INTERNAL_SERVER_ERROR)
		}
	}
}

#[derive(Debug, serde::Deserialize)]
struct PaymentMethodQuery {
	/// always true
	next: bool,
}

#[derive(Debug, serde::Serialize)]
struct PaymentMethodResponse {
	/// Url that the website will open in a new tab
	url: String,
}

async fn payment_method(
	State(global): State<Arc<Global>>,
	Path(target): Path<TargetUser>,
	Query(query): Query<PaymentMethodQuery>,
	auth_session: Option<Extension<AuthSession>>,
) -> Result<Json<PaymentMethodResponse>, ApiError> {
	let user = match target {
		TargetUser::Me => auth_session.ok_or(ApiError::UNAUTHORIZED)?.user_id(),
		TargetUser::Other(id) => id,
	};

	// TODO: Do we need this?
	Err(ApiError::NOT_IMPLEMENTED)
}

async fn products(State(global): State<Arc<Global>>) -> Result<Json<Vec<types::Product>>, ApiError> {
	let products: Vec<SubscriptionProduct> = SubscriptionProduct::collection(&global.db)
		.find(filter::filter! {
			SubscriptionProduct {}
		})
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to query subscription products");
			ApiError::INTERNAL_SERVER_ERROR
		})?
		.try_collect()
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to collect subscription products");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

	let plans = products.iter().cloned().map(Into::into).collect();

	let current_paints = products
		.into_iter()
		.flat_map(|p| p.benefits)
		.filter(|b| match &b.condition {
			SubscriptionBenefitCondition::TimePeriod(time_period) => {
				time_period.start <= chrono::Utc::now() && time_period.end > chrono::Utc::now()
			}
			_ => false,
		})
		.filter_map(|b| match b.entitlement {
			EntitlementEdgeKind::Paint { paint_id } => Some(paint_id),
			_ => None,
		})
		.collect();

	Ok(Json(vec![types::Product {
		name: "subscription".to_string(),
		plans,
		current_paints,
	}]))
}

#[derive(Debug, serde::Deserialize)]
struct RedeemRequest {
	code: String,
}

#[derive(Debug, serde::Serialize)]
struct RedeemResponse {
	/// Url that the website will open
	authorize_url: Option<String>,
	/// list of ids of cosmetics that the user received
	items: Vec<String>,
}

async fn redeem(
	State(global): State<Arc<Global>>,
	Json(body): Json<RedeemRequest>,
	auth_session: Option<Extension<AuthSession>>,
) -> Result<Json<RedeemResponse>, ApiError> {
	let auth_session = auth_session.ok_or(ApiError::UNAUTHORIZED)?;

	let res = with_transaction(&global, |mut tx| async move {
		let code = tx
			.find_one_and_update(
				filter::filter! {
					RedeemCode {
						code: body.code,
						#[query(selector = "gt")]
						remaining_uses: 0,
						active_period: TimePeriod {
							#[query(selector = "lt")]
							start: chrono::Utc::now(),
							#[query(selector = "gt")]
							end: chrono::Utc::now(),
						},
					}
				},
				update::update! {
					#[query(inc)]
					RedeemCode {
						remaining_uses: -1,
					},
				},
				None,
			)
			.await?;

		let Some(code) = code else {
			return Err(TransactionError::custom(ApiError::NOT_FOUND));
		};

		for effect in code.effects {
			match effect {
				CodeEffect::Entitlement { edge } => {
					tx.insert_one(
						EntitlementEdge {
							id: EntitlementEdgeId {
								from: EntitlementEdgeKind::User {
									user_id: auth_session.user_id(),
								},
								to: edge,
								managed_by: Some(EntitlementEdgeManagedBy::RedeemCode { redeem_code_id: code.id }),
							},
						},
						None,
					)
					.await?;
				}
				CodeEffect::SubscriptionProduct { id, trial_days } => {
					// TODO: implement
				}
			}
		}

		Ok(())
	})
	.await;

	match res {
		Ok(_) => Ok(StatusCode::OK),
		Err(TransactionError::Custom(e)) => Err(e),
		Err(e) => {
			tracing::error!(error = %e, "transaction failed");
			Err(ApiError::INTERNAL_SERVER_ERROR)
		}
	}
}
