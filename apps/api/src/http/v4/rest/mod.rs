use std::sync::Arc;

use axum::{routing::post, Router};

use crate::global::Global;

mod stripe_webhooks;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/stripe/webhook", post(stripe_webhooks::handle))
}
