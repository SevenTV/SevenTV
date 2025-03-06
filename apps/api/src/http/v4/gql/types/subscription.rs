use std::sync::Arc;

use async_graphql::Context;
use shared::database::product::codes::RedeemCodeId;
use shared::database::product::subscription::{SubscriptionId, SubscriptionPeriodId};
use shared::database::product::{InvoiceId, StripeProductId};
use shared::database::user::UserId;

use super::User;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v4::gql::types::{SubscriptionProduct, SubscriptionProductVariant};

#[derive(async_graphql::SimpleObject)]
pub struct Subscription {
	pub id: SubscriptionId,
	pub state: SubscriptionState,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub created_at: chrono::DateTime<chrono::Utc>,
	pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<shared::database::product::subscription::Subscription> for Subscription {
	fn from(value: shared::database::product::subscription::Subscription) -> Self {
		Self {
			id: value.id,
			state: value.state.into(),
			updated_at: value.updated_at,
			created_at: value.created_at,
			ended_at: value.ended_at,
			search_updated_at: value.search_updated_at,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, async_graphql::Enum)]
pub enum SubscriptionState {
	Active,
	CancelAtEnd,
	Ended,
}

impl From<shared::database::product::subscription::SubscriptionState> for SubscriptionState {
	fn from(value: shared::database::product::subscription::SubscriptionState) -> Self {
		match value {
			shared::database::product::subscription::SubscriptionState::Active => SubscriptionState::Active,
			shared::database::product::subscription::SubscriptionState::CancelAtEnd => SubscriptionState::CancelAtEnd,
			shared::database::product::subscription::SubscriptionState::Ended => SubscriptionState::Ended,
		}
	}
}

#[derive(async_graphql::SimpleObject, Clone)]
pub struct ProviderSubscriptionId {
	pub provider: SubscriptionProvider,
	pub id: String,
}

impl From<shared::database::product::subscription::ProviderSubscriptionId> for ProviderSubscriptionId {
	fn from(value: shared::database::product::subscription::ProviderSubscriptionId) -> Self {
		match value {
			shared::database::product::subscription::ProviderSubscriptionId::Stripe(id) => ProviderSubscriptionId {
				provider: SubscriptionProvider::Stripe,
				id: id.to_string(),
			},
			shared::database::product::subscription::ProviderSubscriptionId::Paypal(id) => ProviderSubscriptionId {
				provider: SubscriptionProvider::PayPal,
				id,
			},
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, async_graphql::Enum)]
pub enum SubscriptionProvider {
	Stripe,
	PayPal,
}

#[derive(async_graphql::Union, Clone)]
pub enum SubscriptionPeriodCreatedBy {
	RedeemCode(SubscriptionPeriodCreatedByRedeemCode),
	Invoice(SubscriptionPeriodCreatedByInvoice),
	System(SubscriptionPeriodCreatedBySystem),
}

impl From<shared::database::product::subscription::SubscriptionPeriodCreatedBy> for SubscriptionPeriodCreatedBy {
	fn from(value: shared::database::product::subscription::SubscriptionPeriodCreatedBy) -> Self {
		match value {
			shared::database::product::subscription::SubscriptionPeriodCreatedBy::RedeemCode { redeem_code_id } => {
				SubscriptionPeriodCreatedBy::RedeemCode(SubscriptionPeriodCreatedByRedeemCode { redeem_code_id })
			}
			shared::database::product::subscription::SubscriptionPeriodCreatedBy::Invoice { invoice_id } => {
				SubscriptionPeriodCreatedBy::Invoice(SubscriptionPeriodCreatedByInvoice { invoice_id })
			}
			shared::database::product::subscription::SubscriptionPeriodCreatedBy::System { reason } => {
				SubscriptionPeriodCreatedBy::System(SubscriptionPeriodCreatedBySystem { reason })
			}
		}
	}
}

#[derive(async_graphql::SimpleObject, Clone)]
pub struct SubscriptionPeriodCreatedByRedeemCode {
	pub redeem_code_id: RedeemCodeId,
}

#[derive(async_graphql::SimpleObject, Clone)]
pub struct SubscriptionPeriodCreatedByInvoice {
	pub invoice_id: InvoiceId,
}

#[derive(async_graphql::SimpleObject, Clone)]
pub struct SubscriptionPeriodCreatedBySystem {
	pub reason: Option<String>,
}

#[derive(async_graphql::SimpleObject, Clone)]
#[graphql(complex)]
pub struct SubscriptionPeriod {
	pub id: SubscriptionPeriodId,
	pub subscription_id: SubscriptionId,
	pub provider_id: Option<ProviderSubscriptionId>,
	pub product_id: StripeProductId,
	pub start: chrono::DateTime<chrono::Utc>,
	pub end: chrono::DateTime<chrono::Utc>,
	pub is_trial: bool,
	pub auto_renew: bool,
	pub gifted_by_id: Option<UserId>,
	pub created_by: SubscriptionPeriodCreatedBy,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_graphql::ComplexObject]
impl SubscriptionPeriod {
	#[tracing::instrument(skip_all, name = "SubscriptionPeriod::gifted_by")]
	async fn gifted_by(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		let Some(gifted_by_id) = self.gifted_by_id else {
			return Ok(None);
		};

		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_loader
			.load_fast(global, gifted_by_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(user.map(Into::into))
	}

	#[tracing::instrument(skip_all, name = "SubscriptionPeriod::subscription")]
	async fn subscription(&self, ctx: &Context<'_>) -> Result<Subscription, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let subscription = global
			.subscription_by_id_loader
			.load(self.subscription_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription"))?
			.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "subscription not found"))?;

		Ok(subscription.into())
	}

	#[tracing::instrument(skip_all, name = "SubscriptionPeriod::subscription_product")]
	async fn subscription_product(&self, ctx: &Context<'_>) -> Result<SubscriptionProduct, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let product = global
			.subscription_product_by_id_loader
			.load(self.subscription_id.product_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription product"))?
			.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "subscription product not found"))?;

		Ok(product.into())
	}

	#[tracing::instrument(skip_all, name = "SubscriptionPeriod::subscription_product_variant")]
	async fn subscription_product_variant(&self, ctx: &Context<'_>) -> Result<SubscriptionProductVariant, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let product = global
			.subscription_product_by_id_loader
			.load(self.subscription_id.product_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription product"))?
			.ok_or(ApiError::not_found(ApiErrorCode::LoadError, "subscription product not found"))?;

		let variant = product
			.variants
			.into_iter()
			.find(|v| v.id == self.product_id)
			.ok_or_else(|| {
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription product variant")
			})?;

		Ok(SubscriptionProductVariant::from_db(variant, product.default_currency))
	}
}

impl From<shared::database::product::subscription::SubscriptionPeriod> for SubscriptionPeriod {
	fn from(value: shared::database::product::subscription::SubscriptionPeriod) -> Self {
		Self {
			id: value.id,
			subscription_id: value.subscription_id,
			provider_id: value.provider_id.map(Into::into),
			product_id: value.product_id,
			start: value.start,
			end: value.end,
			is_trial: value.is_trial,
			auto_renew: value.auto_renew,
			gifted_by_id: value.gifted_by,
			created_by: value.created_by.into(),
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		}
	}
}
