use std::sync::Arc;

use async_graphql::Context;
use shared::database::product::codes::RedeemCodeId;
use shared::database::product::special_event::SpecialEventId;
use shared::database::product::SubscriptionProductId;
use shared::database::user::UserId;

use super::{EntitlementNodeAny, SpecialEvent, SubscriptionProduct, TimePeriod, User};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct RedeemCode {
	pub id: RedeemCodeId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub code: String,
	pub remaining_uses: i32,
	pub active_period: Option<TimePeriod>,
	pub subscription_effect: Option<RedeemCodeSubscriptionEffect>,
	pub created_by_id: UserId,
	pub effect: CodeEffect,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_graphql::ComplexObject]
impl RedeemCode {
	#[tracing::instrument(skip_all, name = "RedeemCode::created_at")]
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.timestamp()
	}

	#[tracing::instrument(skip_all, name = "RedeemCode::created_by")]
	async fn created_by(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_loader
			.load_fast(global, self.created_by_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(user.map(Into::into))
	}
}

impl From<shared::database::product::codes::RedeemCode> for RedeemCode {
	fn from(code: shared::database::product::codes::RedeemCode) -> Self {
		RedeemCode {
			id: code.id,
			name: code.name,
			description: code.description,
			tags: code.tags,
			code: code.code,
			remaining_uses: code.remaining_uses,
			active_period: code.active_period.map(Into::into),
			subscription_effect: code.subscription_effect.map(Into::into),
			created_by_id: code.created_by,
			effect: code.effect.into(),
			updated_at: code.updated_at,
			search_updated_at: code.search_updated_at,
		}
	}
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct RedeemCodeSubscriptionEffect {
	pub id: SubscriptionProductId,
	pub trial_days: Option<i32>,
	pub no_redirect_to_stripe: bool,
}

impl From<shared::database::product::codes::RedeemCodeSubscriptionEffect> for RedeemCodeSubscriptionEffect {
	fn from(effect: shared::database::product::codes::RedeemCodeSubscriptionEffect) -> Self {
		RedeemCodeSubscriptionEffect {
			id: effect.id,
			trial_days: effect.trial_days,
			no_redirect_to_stripe: effect.no_redirect_to_stripe,
		}
	}
}

#[async_graphql::ComplexObject]
impl RedeemCodeSubscriptionEffect {
	#[tracing::instrument(skip_all, name = "RedeemCodeSubscriptionEffect::subscription_product")]
	async fn subscription_product(&self, ctx: &Context<'_>) -> Result<Option<SubscriptionProduct>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let product =
			global.subscription_product_by_id_loader.load(self.id).await.map_err(|_| {
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription product")
			})?;

		Ok(product.map(Into::into))
	}
}

#[derive(async_graphql::Union)]
pub enum CodeEffect {
	DirectEntitlement(CodeEffectDirectEntitlement),
	SpecialEvent(CodeEffectSpecialEvent),
}

impl From<shared::database::product::codes::CodeEffect> for CodeEffect {
	fn from(effect: shared::database::product::codes::CodeEffect) -> Self {
		match effect {
			shared::database::product::codes::CodeEffect::DirectEntitlement { entitlements } => {
				CodeEffect::DirectEntitlement(CodeEffectDirectEntitlement {
					entitlements: entitlements.into_iter().map(|e| EntitlementNodeAny::from_db(&e)).collect(),
				})
			}
			shared::database::product::codes::CodeEffect::SpecialEvent { special_event_id } => {
				CodeEffect::SpecialEvent(CodeEffectSpecialEvent { special_event_id })
			}
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct CodeEffectDirectEntitlement {
	pub entitlements: Vec<EntitlementNodeAny>,
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct CodeEffectSpecialEvent {
	pub special_event_id: SpecialEventId,
}

#[async_graphql::ComplexObject]
impl CodeEffectSpecialEvent {
	#[tracing::instrument(skip_all, name = "CodeEffectSpecialEvent::special_event")]
	async fn special_event(&self, ctx: &Context<'_>) -> Result<Option<SpecialEvent>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let special_event = global
			.special_event_by_id_loader
			.load(self.special_event_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load special event"))?;

		Ok(special_event.map(Into::into))
	}
}
