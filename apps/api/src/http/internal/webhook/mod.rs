use std::sync::Arc;

use axum::Router;

use crate::global::Global;

mod paypal;
mod stripe;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.nest("/paypal", paypal::routes())
		.nest("/stripe", stripe::routes())
}
