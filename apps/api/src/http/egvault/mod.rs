use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, OnceLock};

use axum::routing::{get, patch, post};
use axum::Router;
use shared::database::product::CustomerId;
use shared::database::queries::{filter, update};
use shared::database::user::{User, UserId};
use shared::database::MongoCollection;
use subscribe::Prefill;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::stripe_client::StripeClient;

mod cancel;
pub mod metadata;
mod payment_method;
mod products;
pub mod redeem;
mod subscribe;
mod subscription;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/subscriptions", post(subscribe::subscribe))
		.route(
			"/subscriptions/:target",
			get(subscription::subscription).delete(cancel::cancel_subscription),
		)
		.route("/subscriptions/:target/reactivate", post(cancel::reactivate_subscription))
		.route("/subscriptions/:target/payment-method", patch(payment_method::payment_method))
		.route("/products", get(products::products))
		.route("/redeem", post(redeem::redeem))
}

#[derive(Debug, Clone)]
pub enum EgVaultMutexKey {
	User(UserId),
}

impl std::fmt::Display for EgVaultMutexKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::User(user_id) => write!(f, "mutex:egvault:user:{user_id}"),
		}
	}
}

pub enum CheckoutProduct {
	Price(stripe::PriceId),
	Gift(stripe::ProductId),
}

fn all_of_them() -> Vec<stripe::CreateCheckoutSessionPaymentMethodTypes> {
	vec![
		stripe::CreateCheckoutSessionPaymentMethodTypes::AcssDebit,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Affirm,
		stripe::CreateCheckoutSessionPaymentMethodTypes::AfterpayClearpay,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Alipay,
		stripe::CreateCheckoutSessionPaymentMethodTypes::AuBecsDebit,
		stripe::CreateCheckoutSessionPaymentMethodTypes::BacsDebit,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Bancontact,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Blik,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Boleto,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Card,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Cashapp,
		stripe::CreateCheckoutSessionPaymentMethodTypes::CustomerBalance,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Eps,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Fpx,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Giropay,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Grabpay,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Ideal,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Klarna,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Konbini,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Link,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Oxxo,
		stripe::CreateCheckoutSessionPaymentMethodTypes::P24,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Paynow,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Paypal,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Pix,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Promptpay,
		stripe::CreateCheckoutSessionPaymentMethodTypes::RevolutPay,
		stripe::CreateCheckoutSessionPaymentMethodTypes::SepaDebit,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Sofort,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Swish,
		stripe::CreateCheckoutSessionPaymentMethodTypes::UsBankAccount,
		stripe::CreateCheckoutSessionPaymentMethodTypes::WechatPay,
		stripe::CreateCheckoutSessionPaymentMethodTypes::Zip,
	]
}

async fn create_checkout_session_params(
	global: &Arc<Global>,
	ip: std::net::IpAddr,
	customer_id: CustomerId,
	product_id: CheckoutProduct,
	default_currency: stripe::Currency,
	currency_prices: &HashMap<stripe::Currency, i64>,
) -> stripe::CreateCheckoutSession<'static> {
	// cursed solution but the ownership has to stay somewhere
	static SUCCESS_URL: OnceLock<String> = OnceLock::new();
	static CANCEL_URL: OnceLock<String> = OnceLock::new();

	let success_url = SUCCESS_URL.get_or_init(|| {
		global
			.config
			.api
			.website_origin
			.join("/subscribe/complete?with_provider=stripe")
			.unwrap()
			.to_string()
	});
	let cancel_url = CANCEL_URL.get_or_init(|| {
		global
			.config
			.api
			.website_origin
			.join("/subscribe/cancel?with_provider=stripe")
			.unwrap()
			.to_string()
	});

	let mut currency = default_currency;

	if let Some(country_code) = global.geoip().and_then(|g| g.lookup(ip)).and_then(|c| c.iso_code) {
		if let Ok(Some(global)) = global.global_config_loader.load(()).await {
			if let Some(currency_override) = global.country_currency_overrides.get(country_code) {
				currency = *currency_override;
			}
		}
	}

	let line = match product_id {
		CheckoutProduct::Gift(gift_id) => stripe::CreateCheckoutSessionLineItems {
			price_data: Some(stripe::CreateCheckoutSessionLineItemsPriceData {
				product: Some(gift_id.to_string()),
				unit_amount: currency_prices.get(&currency).copied(),
				currency,
				..Default::default()
			}),
			quantity: Some(1),
			..Default::default()
		},
		CheckoutProduct::Price(price_id) => stripe::CreateCheckoutSessionLineItems {
			price: Some(price_id.to_string()),
			quantity: Some(1),
			..Default::default()
		},
	};

	stripe::CreateCheckoutSession {
		line_items: Some(vec![line]),
		customer_update: Some(stripe::CreateCheckoutSessionCustomerUpdate {
			address: Some(stripe::CreateCheckoutSessionCustomerUpdateAddress::Auto),
			..Default::default()
		}),
		automatic_tax: Some(stripe::CreateCheckoutSessionAutomaticTax {
			enabled: true,
			..Default::default()
		}),
		currency: currency_prices
			.contains_key(&currency)
			.then_some(currency)
			.or_else(|| currency_prices.contains_key(&currency).then_some(default_currency)),
		customer: Some(customer_id.into()),
		// expire the session 4 hours from now so we can restore unused redeem codes in the checkout.session.expired handler
		expires_at: Some((chrono::Utc::now() + chrono::Duration::hours(4)).timestamp()),
		success_url: Some(success_url),
		cancel_url: Some(cancel_url),
		payment_method_types: Some(all_of_them()),
		..Default::default()
	}
}

async fn find_customer(global: &Arc<Global>, user_id: UserId) -> Result<Option<CustomerId>, ApiError> {
	let mut query = vec![format!("metadata[\"USER_ID\"]:\"{}\"", user_id)];

	// Ensure the search finds users with old object ids too
	if let Some(oid) = user_id.as_object_id() {
		query.push(format!("metadata[\"USER_ID\"]:\"{}\"", oid));
	}

	let query = query.join(" OR ");

	// This doesnt have to be safe because it is a read only operation
	let customer = stripe::Customer::search(
		global.stripe_client.client().await.deref(),
		stripe::CustomerSearchParams {
			query,
			limit: Some(1),
			page: None,
			expand: &[],
		},
	)
	.await
	.map_err(|e| {
		tracing::error!(error = %e, "failed to search customer");
		ApiError::internal_server_error(ApiErrorCode::StripeError, "failed to search customer")
	})?;

	let customer_id = customer.data.into_iter().next().map(|c| c.id.into());

	Ok(customer_id)
}

async fn find_or_create_customer(
	global: &Arc<Global>,
	stripe_client: StripeClient,
	user_id: UserId,
	prefill: Option<Prefill>,
) -> Result<CustomerId, ApiError> {
	let id = match find_customer(global, user_id).await? {
		Some(id) => id,
		None => {
			// no customer found, create one

			let name = prefill.as_ref().map(|p| format!("{} {}", p.first_name, p.last_name));

			let customer = stripe::Customer::create(
				stripe_client.deref(),
				stripe::CreateCustomer {
					email: prefill.map(|p| p.email).as_deref(),
					name: name.as_deref(),
					metadata: Some([("USER_ID".to_string(), user_id.to_string())].into()),
					..Default::default()
				},
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to create customer");
				ApiError::internal_server_error(ApiErrorCode::StripeError, "failed to create customer")
			})?;

			customer.id.into()
		}
	};

	User::collection(&global.db)
		.update_one(
			filter::filter! {
				User {
					#[query(rename = "_id")]
					id: user_id,
				}
			},
			update::update! {
				#[query(set)]
				User {
					stripe_customer_id: &id,
					updated_at: chrono::Utc::now(),
					search_updated_at: &None,
				}
			},
		)
		.await
		.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to update user"))?;

	Ok(id)
}
