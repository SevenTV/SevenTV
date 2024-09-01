use std::sync::Arc;

use axum::routing::post;
use axum::Router;

use crate::global::Global;

mod paypal;
mod stripe;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/stripe", post(stripe::handle))
		.route("/paypal", post(paypal::handle))
}
