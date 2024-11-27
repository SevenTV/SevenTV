use std::sync::Arc;

use axum::routing::{get, patch, post};
use axum::Router;

use crate::global::Global;

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
