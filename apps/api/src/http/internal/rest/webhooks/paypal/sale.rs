use std::ops::Deref;
use std::sync::Arc;

use bson::doc;
use mongodb::options::{FindOneOptions, UpdateOptions};
use shared::database::product::invoice::{Invoice, InvoiceStatus};
use shared::database::product::subscription::{
	ProviderSubscriptionId, SubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy, SubscriptionPeriodId,
};
use shared::database::product::{CustomerId, InvoiceId, SubscriptionProduct, SubscriptionProductVariant};
use shared::database::queries::{filter, update};
use shared::database::user::User;
use stripe::{CreateInvoice, CreateInvoiceItem, FinalizeInvoiceParams};
use tracing::Instrument;

use super::types;
use crate::global::Global;
use crate::http::egvault::metadata::{CustomerMetadata, InvoiceMetadata, StripeMetadata};
use crate::http::error::{ApiError, ApiErrorCode};
use crate::paypal_api;
use crate::stripe_client::SafeStripeClient;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

#[derive(Debug, Clone)]
pub enum StripeRequest {
	CreateCustomer,
	CreateInvoice,
	CreateInvoiceItem,
	FinalizeInvoice,
	VoidInvoice,
}

impl std::fmt::Display for StripeRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::CreateCustomer => write!(f, "create_customer"),
			Self::CreateInvoice => write!(f, "create_invoice"),
			Self::CreateInvoiceItem => write!(f, "add_invoice_item"),
			Self::FinalizeInvoice => write!(f, "finalize_invoice"),
			Self::VoidInvoice => write!(f, "void_invoice"),
		}
	}
}

#[tracing::instrument(skip_all, name = "paypal::sale::completed")]
pub async fn completed(
	global: &Arc<Global>,
	stripe_client: SafeStripeClient<StripeRequest>,
	mut tx: TransactionSession<'_, ApiError>,
	sale: types::Sale,
) -> TransactionResult<Option<SubscriptionId>, ApiError> {
	let Some(provider_id) = sale.billing_agreement_id else {
		// sale isn't related to a subscription
		return Ok(None);
	};

	let Some(period) = tx
		.find_one(
			filter::filter! {
				SubscriptionPeriod {
					#[query(serde)]
					provider_id: Some(ProviderSubscriptionId::Paypal(provider_id.clone())),
				}
			},
			FindOneOptions::builder().sort(doc! { "start": -1 }).build(),
		)
		.await?
	else {
		// no user found
		tracing::warn!(provider_id = %provider_id, "user for paypal subscription not found");
		return Ok(None);
	};

	let user = tx
		.find_one(
			filter::filter! {
				User {
					#[query(rename = "_id")]
					id: period.subscription_id.user_id,
				}
			},
			None,
		)
		.await?
		.ok_or_else(|| {
			tracing::warn!(user_id = %period.subscription_id.user_id, "user not found");
			TransactionError::Custom(ApiError::internal_server_error(ApiErrorCode::LoadError, "user not found"))
		})?;

	// retrieve the paypal subscription
	let api_key = paypal_api::api_key(global).await.map_err(TransactionError::Custom)?;

	let paypal_sub: types::Subscription = async {
		global
			.http_client
			.get(format!("https://api.paypal.com/v1/billing/subscriptions/{provider_id}"))
			.bearer_auth(api_key)
			.send()
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to retrieve paypal subscription");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::PaypalError,
					"failed to retrieve paypal subscription",
				))
			})?
			.json()
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to parse paypal subscription");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::PaypalError,
					"failed to parse paypal subscription",
				))
			})
	}
	.instrument(tracing::info_span!("retrieve_paypal_subscription"))
	.await?;

	// get or create the stripe customer
	let customer_id = match user.stripe_customer_id {
		Some(id) => id,
		None => {
			// no stripe customer yet
			let name = paypal_sub.subscriber.name.and_then(|n| match (n.given_name, n.surname) {
				(Some(given), Some(surname)) => Some(format!("{given} {surname}")),
				(Some(given), None) => Some(given),
				(None, Some(surname)) => Some(surname),
				(None, None) => None,
			});

			let phone = paypal_sub
				.subscriber
				.phone
				.and_then(|p| p.phone_number)
				.and_then(|n| n.national_number);

			let address = paypal_sub.subscriber.shipping_address.map(|a| stripe::Address {
				city: a.admin_area_1,
				country: a.country_code,
				line1: a.address_line_1,
				line2: a.address_line_2,
				postal_code: a.postal_code,
				state: a.admin_area_2,
			});

			let customer = stripe::Customer::create(
				stripe_client.client(StripeRequest::CreateCustomer).await.deref(),
				stripe::CreateCustomer {
					name: name.as_deref(),
					email: paypal_sub.subscriber.email_address.as_deref(),
					phone: phone.as_deref(),
					address,
					description: Some("Legacy PayPal customer. Real payments will be handled by PayPal."),
					metadata: Some(
						CustomerMetadata {
							user_id: user.id,
							paypal_id: Some(paypal_sub.subscriber.payer_id),
						}
						.to_stripe(),
					),
					..Default::default()
				},
			)
			.instrument(tracing::info_span!("create_customer"))
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to create stripe customer");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to create stripe customer",
				))
			})?;

			let customer_id: CustomerId = customer.id.into();

			tx.update_one(
				filter::filter! {
					User {
						#[query(rename = "_id")]
						id: user.id,
					}
				},
				update::update! {
					#[query(set)]
					User {
						stripe_customer_id: Some(&customer_id),
						updated_at: chrono::Utc::now(),
						search_updated_at: &None,
					}
				},
				None,
			)
			.await?;

			customer_id
		}
	};

	let Some(product) = tx
		.find_one(
			filter::filter! {
				SubscriptionProduct {
					#[query(flatten)]
					variants: SubscriptionProductVariant {
						paypal_id: Some(&paypal_sub.plan_id),
						active: true,
					}
				}
			},
			None,
		)
		.await?
	else {
		// no product found
		tracing::warn!(paypal_id = %paypal_sub.plan_id, "product not found");
		return Ok(None);
	};

	let stripe_product = product
		.variants
		.into_iter()
		.find(|v| v.paypal_id.as_ref().is_some_and(|p| p == &paypal_sub.plan_id))
		.unwrap();

	let invoice = stripe::Invoice::create(
		stripe_client.client(StripeRequest::CreateInvoice).await.deref(),
		CreateInvoice {
			customer: Some(customer_id.clone().into()),
			auto_advance: Some(false),
			description: Some("Legacy PayPal invoice. Real payments will be handled by PayPal."),
			metadata: Some(
				InvoiceMetadata::PaypalLegacy {
					paypal_id: sale.id.clone(),
				}
				.to_stripe(),
			),
			..Default::default()
		},
	)
	.instrument(tracing::info_span!("create_invoice"))
	.await
	.map_err(|e| {
		tracing::error!(error = %e, "failed to create invoice");
		TransactionError::Custom(ApiError::internal_server_error(
			ApiErrorCode::StripeError,
			"failed to create invoice",
		))
	})?;

	let mut params = CreateInvoiceItem::new(customer_id.clone().into());
	params.invoice = Some(invoice.id.clone());
	params.price_data = Some(stripe::InvoiceItemPriceData {
		currency: sale.amount.currency,
		product: product.provider_id.to_string(),
		unit_amount_decimal: Some(sale.amount.total.replace('.', "")),
		..Default::default()
	});

	stripe::InvoiceItem::create(stripe_client.client(StripeRequest::CreateInvoiceItem).await.deref(), params)
		.instrument(tracing::info_span!("create_invoice_item"))
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to create invoice item");
			TransactionError::Custom(ApiError::internal_server_error(
				ApiErrorCode::StripeError,
				"failed to create invoice item",
			))
		})?;

	let mut invoice = stripe::Invoice::finalize(
		stripe_client.client(StripeRequest::FinalizeInvoice).await.deref(),
		&invoice.id,
		FinalizeInvoiceParams {
			auto_advance: Some(false),
		},
	)
	.instrument(tracing::info_span!("finalize_invoice"))
	.await
	.map_err(|e| {
		tracing::error!(error = %e, "failed to finalize invoice");
		TransactionError::Custom(ApiError::internal_server_error(
			ApiErrorCode::StripeError,
			"failed to finalize invoice",
		))
	})?;

	// void the invoice
	if invoice.status.is_some_and(|s| s == stripe::InvoiceStatus::Open) {
		invoice = stripe::Invoice::void(stripe_client.client(StripeRequest::VoidInvoice).await.deref(), &invoice.id)
			.instrument(tracing::info_span!("void_invoice"))
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to void invoice");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to void invoice",
				))
			})?;
	}

	let invoice_id: InvoiceId = invoice.id.into();

	let status: InvoiceStatus = invoice
		.status
		.ok_or_else(|| {
			tracing::error!("invoice status is missing");
			TransactionError::Custom(ApiError::internal_server_error(
				ApiErrorCode::StripeError,
				"invoice status is missing",
			))
		})?
		.into();

	let created_at = chrono::DateTime::from_timestamp(invoice.created.unwrap_or_default(), 0).ok_or_else(|| {
		tracing::error!("invoice created_at is missing");
		TransactionError::Custom(ApiError::internal_server_error(
			ApiErrorCode::StripeError,
			"invoice created_at is missing",
		))
	})?;

	tx.update_one(
		filter::filter! {
			Invoice {
				#[query(rename = "_id")]
				id: &invoice_id,
			}
		},
		update::update! {
			#[query(set_on_insert)]
			Invoice {
				#[query(rename = "_id")]
				id: &invoice_id,
				created_at,
			},
			#[query(set)]
			Invoice {
				items: vec![stripe_product.id.clone()],
				customer_id,
				user_id: user.id,
				paypal_payment_id: Some(sale.id.clone()),
				#[query(serde)]
				status,
				failed: false,
				refunded: false,
				#[query(serde)]
				disputed: &None,
				updated_at: created_at,
				search_updated_at: &None,
			}
		},
		Some(UpdateOptions::builder().upsert(true).build()),
	)
	.await?;

	if let Some(next_billing_time) = paypal_sub.billing_info.next_billing_time {
		let subscription_id = SubscriptionId {
			user_id: user.id,
			product_id: product.id,
		};

		tx.insert_one(
			SubscriptionPeriod {
				id: SubscriptionPeriodId::new(),
				subscription_id,
				provider_id: Some(ProviderSubscriptionId::Paypal(provider_id)),
				product_id: stripe_product.id,
				start: paypal_sub
					.billing_info
					.last_payment
					.map(|p| p.time)
					.unwrap_or_else(chrono::Utc::now),
				end: next_billing_time,
				is_trial: false,
				auto_renew: true,
				gifted_by: None,
				created_by: SubscriptionPeriodCreatedBy::Invoice { invoice_id },
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			},
			None,
		)
		.await?;

		return Ok(Some(subscription_id));
	} else {
		// no next billing time
		tracing::warn!(paypal_id = %paypal_sub.id, "next billing time is missing");
	}

	Ok(None)
}

/// Called for `PAYMENT.SALE.REFUNDED`, `PAYMENT.SALE.REVERSED`
///
/// Marks associated invoice as refunded.
#[tracing::instrument(skip_all, name = "paypal::sale::refunded")]
pub async fn refunded(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	refund: types::Refund,
) -> TransactionResult<(), ApiError> {
	tx.update_one(
		filter::filter! {
			Invoice {
				paypal_payment_id: refund.sale_id,
			}
		},
		update::update! {
			#[query(set)]
			Invoice {
				refunded: true,
				updated_at: chrono::Utc::now(),
				search_updated_at: &None,
			}
		},
		None,
	)
	.await?;

	Ok(())
}
