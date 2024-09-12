use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use shared::database::product::{Product, ProductId, SubscriptionProduct, SubscriptionProductVariant};
use shared::database::queries::{filter, update};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::stripe_client::SafeStripeClient;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

// we only have one request
pub type StripeRequest = ();

pub async fn updated(
	_global: &Arc<Global>,
	stripe_client: SafeStripeClient<super::StripeRequest>,
	mut tx: TransactionSession<'_, ApiError>,
	price: stripe::Price,
) -> TransactionResult<(), ApiError> {
	let price = stripe::Price::retrieve(
		stripe_client.client(super::StripeRequest::Price(())).await.deref(),
		&price.id,
		&["currency_options"],
	)
	.await
	.map_err(|e| {
		tracing::error!(error = %e, "failed to retrieve price");
		TransactionError::Custom(ApiError::internal_server_error(
			ApiErrorCode::StripeError,
			"failed to retrieve price",
		))
	})?;

	let product_id: ProductId = price.id.into();

	let active = price.active == Some(true);

	let currency_prices: HashMap<_, _> = price
		.currency_options
		.unwrap_or_default()
		.into_iter()
		.filter_map(|(k, v)| v.unit_amount.map(|a| (k, a)))
		.collect();

	tx.update(
		filter::filter! {
			Product {
				#[query(rename = "_id")]
				id: &product_id,
			}
		},
		update::update! {
			#[query(set)]
			Product {
				active: active,
				#[query(serde)]
				currency_prices: currency_prices.clone(),
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	tx.update(
		filter::filter! {
			SubscriptionProduct {
				#[query(elem_match)]
				variants: SubscriptionProductVariant {
					id: &product_id,
				}
			}
		},
		update::update! {
			#[query(set)]
			SubscriptionProduct {
				#[query(flatten, index = "$")]
				variants: SubscriptionProductVariant {
					active: active,
					#[query(serde)]
					currency_prices: currency_prices,
				},
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	Ok(())
}

pub async fn deleted(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	price: stripe::Price,
) -> TransactionResult<(), ApiError> {
	// Prices can only be deleted if there are no payments associated with them.
	// This means we can safely delete them from our data too.

	let product_id: ProductId = price.id.into();

	tx.update(
		filter::filter! {
			SubscriptionProduct {
				#[query(elem_match)]
				variants: SubscriptionProductVariant {
					id: &product_id,
				}
			}
		},
		update::update! {
			#[query(pull)]
			SubscriptionProduct {
				variants: SubscriptionProductVariant {
					id: &product_id,
				},
			},
			#[query(set)]
			SubscriptionProduct {
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	tx.delete(
		filter::filter! {
			Product {
				#[query(rename = "_id")]
				id: &product_id,
			}
		},
		None,
	)
	.await?;

	Ok(())
}
