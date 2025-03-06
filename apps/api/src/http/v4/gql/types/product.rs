use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use async_graphql::Context;
use shared::database::product::special_event::SpecialEventId;
use shared::database::product::{ProductId, StripeProductId, SubscriptionBenefitId, SubscriptionProductId};
use shared::database::user::UserId;

use super::User;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct Product {
	pub id: ProductId,
	pub provider_id: String,
	pub name: String,
	pub description: Option<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_graphql::ComplexObject]
impl Product {
	#[tracing::instrument(skip_all, name = "Product::created_at")]
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.timestamp()
	}
}

impl From<shared::database::product::Product> for Product {
	fn from(value: shared::database::product::Product) -> Self {
		Self {
			id: value.id,
			provider_id: value.provider_id.to_string(),
			name: value.name,
			description: value.description,
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		}
	}
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct SubscriptionProduct {
	pub id: SubscriptionProductId,
	pub provider_id: String,
	pub name: String,
	pub description: Option<String>,
	pub benefits: Vec<SubscriptionBenefit>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,

	#[graphql(skip)]
	pub default_currency: stripe::Currency,
	#[graphql(skip)]
	pub default_variant_idx: i32,
	#[graphql(skip)]
	pub variants: Vec<shared::database::product::SubscriptionProductVariant>,
}

#[async_graphql::ComplexObject]
impl SubscriptionProduct {
	#[tracing::instrument(skip_all, name = "SubscriptionProduct::default_variant")]
	async fn default_variant(&self) -> Result<SubscriptionProductVariant, ApiError> {
		let mut variant = self.variants.get(self.default_variant_idx as usize).cloned();

		variant
			.take_if(|v| v.active)
			.map(|v| SubscriptionProductVariant::from_db(v, self.default_currency))
			.ok_or(ApiError::internal_server_error(
				ApiErrorCode::LoadError,
				"failed to load default variant",
			))
	}

	#[tracing::instrument(skip_all, name = "SubscriptionProduct::variants")]
	async fn variants(&self) -> Vec<SubscriptionProductVariant> {
		self.variants
			.iter()
			.filter(|v| v.active)
			.map(|v| SubscriptionProductVariant::from_db(v.clone(), self.default_currency))
			.collect()
	}
}

impl From<shared::database::product::SubscriptionProduct> for SubscriptionProduct {
	fn from(value: shared::database::product::SubscriptionProduct) -> Self {
		Self {
			id: value.id,
			provider_id: value.provider_id.to_string(),
			name: value.name,
			description: value.description,
			benefits: value.benefits.into_iter().map(Into::into).collect(),
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
			default_currency: value.default_currency,
			default_variant_idx: value.default_variant_idx,
			variants: value.variants,
		}
	}
}

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct SubscriptionProductVariant {
	pub id: StripeProductId,
	pub paypal_id: Option<String>,
	pub kind: SubscriptionProductKind,

	#[graphql(skip)]
	pub currency_prices: HashMap<stripe::Currency, i64>,
	#[graphql(skip)]
	pub default_currency: stripe::Currency,
}

#[derive(async_graphql::SimpleObject)]
pub struct Price {
	pub currency: String,
	pub amount: i64,
}

#[async_graphql::ComplexObject]
impl SubscriptionProductVariant {
	#[tracing::instrument(skip_all, name = "SubscriptionProductVariant::price")]
	async fn price(&self, ctx: &Context<'_>, preferred_currency: Option<String>) -> Result<Price, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;

		let mut currency = self.default_currency;

		if let Some(country_code) = global.geoip().and_then(|g| g.lookup(session.ip())).and_then(|c| c.iso_code) {
			if let Ok(Some(global)) = global.global_config_loader.load(()).await {
				if let Some(country_currency) = global.country_currency_overrides.get(country_code) {
					currency = *country_currency;
				}
			}
		}

		if let Some(c) = preferred_currency {
			currency = stripe::Currency::from_str(&c)
				.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "invalid currency"))?;
		}

		match self.currency_prices.get(&currency) {
			Some(c) => Ok(Price {
				currency: currency.to_string(),
				amount: *c,
			}),
			None => {
				let amount = self.currency_prices.get(&self.default_currency).copied().ok_or_else(|| {
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load default price")
				})?;

				Ok(Price {
					currency: self.default_currency.to_string(),
					amount,
				})
			}
		}
	}
}

impl SubscriptionProductVariant {
	pub fn from_db(
		variant: shared::database::product::SubscriptionProductVariant,
		default_currency: stripe::Currency,
	) -> Self {
		Self {
			id: variant.id,
			paypal_id: variant.paypal_id,
			kind: variant.kind.into(),
			currency_prices: variant.currency_prices,
			default_currency,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, async_graphql::Enum)]
pub enum SubscriptionProductKind {
	Monthly,
	Yearly,
}

impl From<shared::database::product::SubscriptionProductKind> for SubscriptionProductKind {
	fn from(kind: shared::database::product::SubscriptionProductKind) -> Self {
		match kind {
			shared::database::product::SubscriptionProductKind::Monthly => Self::Monthly,
			shared::database::product::SubscriptionProductKind::Yearly => Self::Yearly,
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct SubscriptionBenefit {
	pub id: SubscriptionBenefitId,
	pub name: String,
	// pub condition: SubscriptionBenefitCondition,
}

impl From<shared::database::product::SubscriptionBenefit> for SubscriptionBenefit {
	fn from(benefit: shared::database::product::SubscriptionBenefit) -> Self {
		Self {
			id: benefit.id,
			name: benefit.name,
			// condition: benefit.condition.into(),
		}
	}
}

// #[derive(async_graphql::SimpleObject)]
// pub struct SubscriptionBenefitCondition {

// }

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct SpecialEvent {
	pub id: SpecialEventId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub created_by_id: UserId,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_graphql::ComplexObject]
impl SpecialEvent {
	#[tracing::instrument(skip_all, name = "SpecialEvent::created_at")]
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.timestamp()
	}

	#[tracing::instrument(skip_all, name = "SpecialEvent::created_by")]
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

impl From<shared::database::product::special_event::SpecialEvent> for SpecialEvent {
	fn from(event: shared::database::product::special_event::SpecialEvent) -> Self {
		Self {
			id: event.id,
			name: event.name,
			description: event.description,
			tags: event.tags,
			created_by_id: event.created_by,
			updated_at: event.updated_at,
			search_updated_at: event.search_updated_at,
		}
	}
}
