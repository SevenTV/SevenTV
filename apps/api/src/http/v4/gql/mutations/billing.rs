use std::ops::Deref;
use std::sync::Arc;

use async_graphql::Context;
use shared::database::product::subscription::{
	ProviderSubscriptionId, Subscription, SubscriptionId, SubscriptionPeriod, SubscriptionState,
};
use shared::database::product::{ProductId, SubscriptionProduct, SubscriptionProductId, SubscriptionProductVariant};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, UserPermission};
use shared::database::user::UserId;
use shared::database::{Id, MongoCollection};

use crate::global::Global;
use crate::http::egvault::metadata::{CheckoutSessionMetadata, InvoiceMetadata, StripeMetadata, SubscriptionMetadata};
use crate::http::egvault::redeem::redeem_code_inner;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::{PermissionGuard, RateLimitGuard};
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::billing::SubscriptionInfo;
use crate::paypal_api;
use crate::stripe_common::{create_checkout_session_params, find_or_create_customer, CheckoutProduct, EgVaultMutexKey};
use crate::sub_refresh_job::SubAge;
use crate::transactions::{transaction_with_mutex, TransactionError};

pub struct BillingMutation {
	pub user_id: UserId,
}

#[derive(async_graphql::SimpleObject)]
pub struct SubscribeResponse {
	pub checkout_url: String,
}

#[derive(async_graphql::SimpleObject)]
pub struct RedeemResponse {
	pub checkout_url: Option<String>,
}

#[async_graphql::Object]
impl BillingMutation {
	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EgVaultSubscribe, 1)")]
	#[tracing::instrument(skip_all, name = "BillingMutation::subscribe")]
	async fn subscribe(&self, ctx: &Context<'_>, variant_id: ProductId) -> Result<SubscribeResponse, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;

		let authed_user = session.user()?;

		if !authed_user.has(UserPermission::Billing) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"this user isn't allowed to use billing features",
			));
		}

		let gift_for = (self.user_id != authed_user.id).then_some(self.user_id);

		let product: SubscriptionProduct = SubscriptionProduct::collection(&global.db)
			.find_one(filter::filter! {
				SubscriptionProduct {
					#[query(flatten)]
					variants: SubscriptionProductVariant {
						#[query(serde)]
						id: &variant_id,
						active: true,
					}
				}
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to find subscription product");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to find subscription product")
			})?
			.ok_or_else(|| ApiError::internal_server_error(ApiErrorCode::LoadError, "subscription product not found"))?;

		let variant = product.variants.into_iter().find(|v| v.id == variant_id && v.active).unwrap();

		let customer_id = match authed_user.stripe_customer_id.clone() {
			Some(id) => id,
			None => {
				// We don't need the safe client here because this won't be retried
				find_or_create_customer(global, global.stripe_client.client().await, authed_user.id, None).await?
			}
		};

		let success_url = global
			.config
			.api
			.beta_website_origin
			.join("/store?success=1")
			.unwrap()
			.to_string();

		let cancel_url = global.config.api.beta_website_origin.join("/store").unwrap().to_string();

		let mut params = create_checkout_session_params(
			global,
			session.ip(),
			customer_id,
			match &gift_for {
				Some(_) => CheckoutProduct::Gift(product.provider_id),
				None => CheckoutProduct::Price(variant.id.0.clone()),
			},
			product.default_currency,
			&variant.currency_prices,
			&success_url,
			&cancel_url,
		)
		.await;

		if let Some(gift_for) = gift_for {
			let receiving_user = global
				.user_loader
				.load_fast(global, gift_for)
				.await
				.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
				.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

			params.mode = Some(stripe::CheckoutSessionMode::Payment);
			params.payment_intent_data = Some(stripe::CreateCheckoutSessionPaymentIntentData {
				description: Some(format!(
					"Gift subscription for {} (7TV:{})",
					receiving_user
						.connections
						.first()
						.map(|c| { format!("{} ({}:{})", c.platform_display_name, c.platform, c.platform_id) })
						.unwrap_or_else(|| "Unknown User".to_owned()),
					receiving_user.id
				)),
				..Default::default()
			});

			params.invoice_creation = Some(stripe::CreateCheckoutSessionInvoiceCreation {
				enabled: true,
				invoice_data: Some(stripe::CreateCheckoutSessionInvoiceCreationInvoiceData {
					metadata: Some(
						InvoiceMetadata::Gift {
							customer_id: authed_user.id,
							user_id: receiving_user.id,
							product_id: variant.id.clone(),
							subscription_product_id: Some(product.id),
						}
						.to_stripe(),
					),
					..Default::default()
				}),
			});

			params.metadata = Some(CheckoutSessionMetadata::Gift.to_stripe());
		} else {
			let is_subscribed = global
				.active_subscription_period_by_user_id_loader
				.load(authed_user.id)
				.await
				.map_err(|()| {
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription period")
				})?
				.is_some();

			if is_subscribed {
				return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "user is already subscribed"));
			}

			params.mode = Some(stripe::CheckoutSessionMode::Subscription);
			params.subscription_data = Some(stripe::CreateCheckoutSessionSubscriptionData {
				metadata: Some(
					SubscriptionMetadata {
						user_id: authed_user.id,
						customer_id: None,
					}
					.to_stripe(),
				),
				..Default::default()
			});

			params.metadata = Some(CheckoutSessionMetadata::Subscription.to_stripe());
		}

		// We don't need the safe client here because this won't be retried
		let session_url = stripe::CheckoutSession::create(global.stripe_client.client().await.deref(), params)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to create checkout session");
				ApiError::internal_server_error(ApiErrorCode::StripeError, "failed to create checkout session")
			})?
			.url
			.ok_or_else(|| {
				ApiError::internal_server_error(ApiErrorCode::StripeError, "failed to create checkout session")
			})?;

		Ok(SubscribeResponse {
			checkout_url: session_url,
		})
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EgVaultPaymentMethod, 1)")]
	#[tracing::instrument(skip_all, name = "BillingMutation::cancel_subscription")]
	async fn cancel_subscription(
		&self,
		ctx: &Context<'_>,
		product_id: SubscriptionProductId,
	) -> Result<SubscriptionInfo, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;

		let auth_user = session.user()?;

		let target_id = self.user_id;

		let target = global
			.user_loader
			.load(global, target_id)
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

		let stripe_client = global.stripe_client.safe(Id::<()>::new()).await;

		let res = transaction_with_mutex(global, Some(EgVaultMutexKey::User(target_id).into()), |mut tx| {
			let global = Arc::clone(global);

			async move {
				let periods = tx
					.find(
						filter::filter! {
							SubscriptionPeriod {
								#[query(flatten)]
								subscription_id: SubscriptionId {
									user_id: target_id,
									product_id: product_id,
								},
							}
						},
						None,
					)
					.await?;

				let active_period = periods
					.iter()
					.find(|p| p.start < chrono::Utc::now() && p.end > chrono::Utc::now())
					.ok_or(TransactionError::Custom(ApiError::not_found(
						ApiErrorCode::BadRequest,
						"subscription not found",
					)))?;

				match &active_period.provider_id {
					Some(ProviderSubscriptionId::Stripe(id)) => {
						stripe::Subscription::update(
							stripe_client.client("update").await.deref(),
							id,
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
									id: active_period.id,
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

				// This would get updated by the sub refresh job eventually but we want it to
				// reflect instantly
				tx.update_one(
					filter::filter! {
						Subscription {
							#[query(rename = "_id", serde)]
							id: active_period.subscription_id,
						}
					},
					update::update! {
						#[query(set)]
						Subscription {
							#[query(serde)]
							state: SubscriptionState::CancelAtEnd,
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					},
					None,
				)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to update subscription");
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::MutationError,
						"failed to update subscription",
					))
				})?;

				let age = SubAge::new(&periods);

				Ok(SubscriptionInfo {
					active_period: Some(active_period.clone().into()),
					total_days: age.days,
				})
			}
		})
		.await;

		match res {
			Ok(info) => Ok(info),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EgVaultSubscribe, 1)")]
	#[tracing::instrument(skip_all, name = "BillingMutation::reactivate_subscription")]
	async fn reactivate_subscription(
		&self,
		ctx: &Context<'_>,
		product_id: SubscriptionProductId,
	) -> Result<SubscriptionInfo, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;

		let auth_user = session.user()?;

		let target_id = self.user_id;

		let target = global
			.user_loader
			.load(global, target_id)
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

		let stripe_client = global.stripe_client.safe(Id::<()>::new()).await;

		let res = transaction_with_mutex(global, Some(EgVaultMutexKey::User(target_id).into()), |mut tx| async move {
			let periods = tx
				.find(
					filter::filter! {
						SubscriptionPeriod {
							#[query(flatten)]
							subscription_id: SubscriptionId {
								user_id: target_id,
								product_id: product_id,
							},
						}
					},
					None,
				)
				.await?;

			let active_period = periods
				.iter()
				.find(|p| p.start < chrono::Utc::now() && p.end > chrono::Utc::now())
				.ok_or_else(|| {
					TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "subscription not found"))
				})?;

			match &active_period.provider_id {
				Some(ProviderSubscriptionId::Stripe(id)) => {
					stripe::Subscription::update(
						stripe_client.client("update").await.deref(),
						id,
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

					// This would get updated by the sub refresh job eventually but we want it to
					// reflect instantly
					tx.update_one(
						filter::filter! {
							Subscription {
								#[query(rename = "_id", serde)]
								id: active_period.subscription_id,
							}
						},
						update::update! {
							#[query(set)]
							Subscription {
								#[query(serde)]
								state: SubscriptionState::Active,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							}
						},
						None,
					)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to update subscription");
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::MutationError,
							"failed to update subscription",
						))
					})?;

					let age = SubAge::new(&periods);

					Ok(SubscriptionInfo {
						active_period: Some(active_period.clone().into()),
						total_days: age.days,
					})
				}
				_ => Err(TransactionError::Custom(ApiError::not_implemented(
					ApiErrorCode::BadRequest,
					"this subscription cannot be reactivated",
				))),
			}
		})
		.await;

		match res {
			Ok(info) => Ok(info),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}

	#[graphql(
		guard = "PermissionGuard::one(UserPermission::Billing).and(RateLimitGuard::new(RateLimitResource::EgVaultRedeem, 1))"
	)]
	async fn redeem_code(
		&self,
		ctx: &Context<'_>,
		#[graphql(validator(min_length = 1, max_length = 24))] code: String,
	) -> Result<RedeemResponse, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;

		let user_id = session
			.user_id()
			.ok_or_else(|| ApiError::unauthorized(ApiErrorCode::LoginRequired, "you are not logged in"))?;

		if self.user_id != user_id {
			return Err(ApiError::bad_request(
				ApiErrorCode::BadRequest,
				"you can only redeem codes for yourself",
			));
		}

		let checkout_url = redeem_code_inner(global, session, code).await?;

		Ok(RedeemResponse { checkout_url })
	}
}
