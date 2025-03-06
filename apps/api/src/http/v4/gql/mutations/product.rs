use std::str::FromStr;
use std::sync::Arc;

use async_graphql::Context;
use shared::database::product::{ProductId, StripeProductId};
use shared::database::role::permissions::AdminPermission;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::PermissionGuard;
use crate::http::v4::gql::types::Product;

#[derive(Default)]
pub struct ProductMutation;

#[derive(async_graphql::InputObject)]
struct CreateProductInput {
	pub provider_id: String,
	pub name: String,
	pub description: Option<String>,
	pub price: i64,
	pub active: bool,
}

#[async_graphql::Object]
impl ProductMutation {
	#[tracing::instrument(skip_all, name = "ProdcutMutation::create")]
	#[graphql(guard = "PermissionGuard::one(AdminPermission::Admin)")]
	async fn create(&self, ctx: &Context<'_>, data: CreateProductInput) -> Result<Product, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let product = shared::database::product::Product {
			id: ProductId::default(),
			provider_id: StripeProductId::from_str(&data.provider_id).unwrap(),
			active: data.active,
			name: data.name,
			description: data.description,
			discount: None,
			extends_subscription: None,
			default_currency: stripe::Currency::EUR,
			currency_prices: [(stripe::Currency::EUR, data.price)].into(),
			created_at: chrono::Utc::now(),
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		};

		shared::database::product::Product::collection(&global.db)
			.insert_one(&product)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "Failed to create");
				ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to create redeem code")
			})?;

		Ok(product.into())
	}
}
