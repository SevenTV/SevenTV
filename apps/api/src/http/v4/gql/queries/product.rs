use std::sync::Arc;

use async_graphql::Context;
use shared::database::product::SubscriptionProductId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v4::gql::types::SubscriptionProduct;

#[derive(Default)]
pub struct ProductQuery;

#[async_graphql::Object]
impl ProductQuery {
	#[tracing::instrument(skip_all, name = "ProductQuery::subscription_products")]
	async fn subscription_products(&self, ctx: &Context<'_>) -> Result<Vec<SubscriptionProduct>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let products = global
			.subscription_products_loader
			.load(())
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription products"))?
			.unwrap_or_default();

		Ok(products.into_iter().map(Into::into).collect())
	}

	#[tracing::instrument(skip_all, name = "ProductQuery::subscription_product")]
	async fn subscription_product(
		&self,
		ctx: &Context<'_>,
		id: SubscriptionProductId,
	) -> Result<Option<SubscriptionProduct>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let product =
			global.subscription_product_by_id_loader.load(id).await.map_err(|_| {
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription product")
			})?;

		Ok(product.map(Into::into))
	}
}
