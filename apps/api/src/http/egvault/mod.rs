use std::sync::Arc;

use axum::routing::{get, patch, post};
use axum::Router;
use shared::database::product::{CustomerId, ProductId};
use shared::database::queries::{filter, update};
use shared::database::user::{User, UserId};
use shared::database::MongoCollection;
use subscribe::Prefill;
use tokio::sync::OnceCell;

use crate::global::Global;
use crate::http::error::ApiError;

mod cancel;
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
		.route("/subscription/:target/reactivate", post(cancel::reactivate_subscription))
		.route("/subscriptions/:target/payment-method", patch(payment_method::payment_method))
		.route("/products", get(products::products))
		.route("/redeem", post(redeem::redeem))
}

async fn create_checkout_session_params<'a>(
	global: &'a Arc<Global>,
	customer_id: CustomerId,
	product_id: Option<&ProductId>,
) -> stripe::CreateCheckoutSession<'a> {
	// cursed solution but the ownership has to stay somewhere
	static SUCCESS_URL: OnceCell<String> = OnceCell::const_new();
	static CANCEL_URL: OnceCell<String> = OnceCell::const_new();

	let success_url = SUCCESS_URL
		.get_or_init(|| async { format!("{}/subscribe/complete?with_provider=stripe", global.config.api.website_origin) })
		.await;
	let cancel_url = CANCEL_URL
		.get_or_init(|| async { format!("{}/subscribe/cancel?with_provider=stripe", global.config.api.website_origin) })
		.await;

	stripe::CreateCheckoutSession {
		line_items: product_id.map(|id| {
			vec![stripe::CreateCheckoutSessionLineItems {
				price: Some(id.to_string()),
				quantity: Some(1),
				..Default::default()
			}]
		}),
		customer_update: Some(stripe::CreateCheckoutSessionCustomerUpdate {
			address: Some(stripe::CreateCheckoutSessionCustomerUpdateAddress::Auto),
			..Default::default()
		}),
		customer: Some(customer_id.into()),
		// expire the session 4 hours from now so we can restore unused redeem codes in the checkout.session.expired handler
		expires_at: Some((chrono::Utc::now() + chrono::Duration::hours(4)).timestamp()),
		success_url: Some(success_url),
		cancel_url: Some(cancel_url),
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

	let customer = stripe::Customer::search(
		&global.stripe_client,
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
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	let customer_id = customer.data.into_iter().next().map(|c| c.id.into());

	Ok(customer_id)
}

async fn find_or_create_customer(
	global: &Arc<Global>,
	user_id: UserId,
	prefill: Option<Prefill>,
) -> Result<CustomerId, ApiError> {
	let id = match find_customer(&global, user_id).await? {
		Some(id) => id,
		None => {
			// no customer found, create one

			let name = prefill.as_ref().map(|p| format!("{} {}", p.first_name, p.last_name));

			let customer = stripe::Customer::create(
				&global.stripe_client,
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
				ApiError::INTERNAL_SERVER_ERROR
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
				}
			},
		)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

	Ok(id)
}