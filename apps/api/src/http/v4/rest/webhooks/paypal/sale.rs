use std::sync::Arc;

use shared::database::{
	product::{
		invoice::Invoice,
		subscription::{PaypalSubscription, ProviderSubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy},
	},
	queries::{filter, update},
};
use stripe::{CreateInvoice, FinalizeInvoiceParams};

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{TransactionError, TransactionResult, TransactionSession},
};

use super::types;

pub async fn completed(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	sale: types::Sale,
) -> TransactionResult<(), ApiError> {
	let Some(subscription_id) = sale.billing_agreement_id else {
		// sale isn't related to a subscription
		return Ok(());
	};

	let Some(pp_sub) = tx
		.find_one(
			filter::filter! {
				PaypalSubscription {
					#[query(rename = "_id")]
					id: subscription_id,
				}
			},
			None,
		)
		.await?
	else {
		// no subscription found
		return Ok(());
	};

	// retrieve the paypal subscription
	let subscription: types::Subscription = global
		.http_client
		.get(format!("https://api.paypal.com/v1/billing/subscriptions/{subscription_id}"))
		.bearer_auth(&global.config.api.paypal.api_key)
		.send()
		.await?
		.json()
		.await?;

	let customer_id = match pp_sub.stripe_customer_id {
		Some(id) => id,
		None => {
			// no stripe customer yet

			let name = subscription.subscriber.name.and_then(|n| match (n.given_name, n.surname) {
				(Some(given), Some(surname)) => Some(format!("{given} {surname}")),
				(Some(given), None) => Some(given),
				(None, Some(surname)) => Some(surname),
				(None, None) => None,
			});

			let phone = subscription
				.subscriber
				.phone
				.and_then(|p| p.phone_number)
				.and_then(|n| n.national_number);

			let address = subscription.subscriber.shipping_address.map(|a| stripe::Address {
				city: a.admin_area_1,
				country: a.country_code,
				line1: a.address_line_1,
				line2: a.address_line_2,
				postal_code: a.postal_code,
				state: a.admin_area_2,
			});

			stripe::Customer::create(
				&global.stripe_client,
				stripe::CreateCustomer {
					name,
					email: subscription.subscriber.email_address,
					phone,
					address,
					description: Some("Legacy PayPal customer. Real payments will be handled by PayPal."),
					metadata: Some(std::iter::once(("paypal_id", subscription.subscriber.payer_id)).collect()),
					..Default::default()
				},
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to create stripe customer");
				TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
			})?
			.id
		}
	};

	tx.update_one(
		filter::filter! {
			PaypalSubscription {
				#[query(rename = "_id")]
				id: subscription_id,
			}
		},
		update::update! {
			#[query(set)]
			PaypalSubscription {
				stripe_customer_id: Some(customer_id),
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	let invoice = stripe::Invoice::create(
		&global.stripe_client,
		CreateInvoice {
			customer: customer_id,
			auto_advance: Some(false),
			description: Some("Legacy PayPal invoice. Real payments will be handled by PayPal."),
			metadata: Some(std::iter::once(("paypal_id", sale.id)).collect()),
			..Default::default()
		},
	)
	.await?;

	stripe::Invoice::finalize(
		&global.stripe_client,
		&invoice.id,
		FinalizeInvoiceParams {
			auto_advance: Some(false),
		},
	)
	.await?;

	let invoice = stripe::Invoice::void(&global.stripe_client, &invoice.id).await?;
	let invoice_id = invoice.id.into();

	tx.insert_one(
		Invoice {
			id: invoice_id.clone(),
			items: vec![pp_sub.product_id],
			customer_id: customer_id.into(),
			user_id: pp_sub.user_id,
			paypal_payment_id: Some(sale.id.clone()),
			status: invoice.status.into(),
			failed: false,
			refunded: false,
			disputed: None,
			created_at: invoice.created.unwrap_or_default(),
			updated_at: invoice.created.unwrap_or_default(),
			search_updated_at: None,
		},
		None,
	)
	.await?;

	if let Some(next_billing_time) = subscription.billing_info.next_billing_time {
		tx.insert_one(
			SubscriptionPeriod {
				id: SubscriptionPeriodId::new(),
				subscription_id: ProviderSubscriptionId::Paypal(subscription_id),
				user_id: pp_sub.user_id,
				start: subscription
					.billing_info
					.last_payment
					.map(|p| p.time)
					.unwrap_or_else(chrono::Utc::now),
				end: next_billing_time,
				is_trial: false,
				created_by: SubscriptionPeriodCreatedBy::Invoice { invoice_id },
				product_ids: vec![pp_sub.product_id],
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			},
			None,
		)
		.await?;
	}

	Ok(())
}

/// Called for `PAYMENT.SALE.REFUNDED`, `PAYMENT.SALE.REVERSED`
///
/// Marks associated invoice as refunded.
pub async fn refunded(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	sale: types::Sale,
) -> TransactionResult<(), ApiError> {
	tx.update_one(
		filter::filter! {
			Invoice {
				paypal_payment_id: sale.id,
			}
		},
		update::update! {
			#[query(set)]
			Invoice {
				refunded: true,
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	Ok(())
}
