use std::sync::Arc;

use axum::{
	extract::State,
	http::StatusCode,
	routing::{get, patch, post},
	Extension, Json, Router,
};

use crate::{
	global::Global,
	http::{
		error::ApiError,
		extract::{Path, Query},
		middleware::auth::AuthSession,
	},
};

use super::{types, users::TargetUser};

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/subscriptions", post(subscribe))
		.route("/subscriptions/:target", get(subscription).delete(cancel_subscription))
		.route("/subscription/:target/reactivate", post(reactivate_subscription))
		.route("/subscription/:target/payment-method", patch(payment_method))
		.route("/products", get(products))
		.route("/redeem", post(redeem))
}

#[derive(Debug, serde::Deserialize)]
struct SubscribeQuery {
	renew_interval: SubscriptionRenewInterval,
	/// only "stripe" allowed
	payment_method: String,
	/// always true
	next: bool,
	gift_for: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SubscribeBody {
	prefill: Prefill,
}

#[derive(Debug, serde::Deserialize)]
struct Prefill {
	first_name: String,
	last_name: String,
	email: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum SubscriptionRenewInterval {
	Monthly,
	Yearly,
}

#[derive(Debug, serde::Serialize)]
struct SubscribeResponse {
	/// Url that the website will open in a new tab
	url: String,
	/// The user id of the user that receives the subscription
	user_id: String,
}

async fn subscribe(
	State(global): State<Arc<Global>>,
	Query(query): Query<SubscribeQuery>,
	auth_session: Option<Extension<AuthSession>>,
	Json(body): Json<SubscribeBody>,
) -> Result<Json<SubscribeResponse>, ApiError> {
	todo!()
}

async fn subscription(
	State(global): State<Arc<Global>>,
	Path(target): Path<TargetUser>,
	auth_session: Option<Extension<AuthSession>>,
) -> Result<Json<types::Subscription>, ApiError> {
	let user = match target {
		TargetUser::Me => auth_session.ok_or(ApiError::UNAUTHORIZED)?.user_id(),
		TargetUser::Other(id) => id,
	};

	todo!()
}

async fn cancel_subscription(
	State(global): State<Arc<Global>>,
	Path(target): Path<TargetUser>,
	auth_session: Option<Extension<AuthSession>>,
) -> Result<StatusCode, ApiError> {
	let user = match target {
		TargetUser::Me => auth_session.ok_or(ApiError::UNAUTHORIZED)?.user_id(),
		TargetUser::Other(id) => id,
	};

	todo!()
}

async fn reactivate_subscription(
	State(global): State<Arc<Global>>,
	Path(target): Path<TargetUser>,
	auth_session: Option<Extension<AuthSession>>,
) -> Result<StatusCode, ApiError> {
	let user = match target {
		TargetUser::Me => auth_session.ok_or(ApiError::UNAUTHORIZED)?.user_id(),
		TargetUser::Other(id) => id,
	};

	todo!()
}

#[derive(Debug, serde::Deserialize)]
struct PaymentMethodQuery {
	/// always true
	next: bool,
}

#[derive(Debug, serde::Serialize)]
struct PaymentMethodResponse {
	/// Url that the website will open in a new tab
	url: String,
}

async fn payment_method(
	State(global): State<Arc<Global>>,
	Path(target): Path<TargetUser>,
	Query(query): Query<PaymentMethodQuery>,
	auth_session: Option<Extension<AuthSession>>,
) -> Result<Json<PaymentMethodResponse>, ApiError> {
	let user = match target {
		TargetUser::Me => auth_session.ok_or(ApiError::UNAUTHORIZED)?.user_id(),
		TargetUser::Other(id) => id,
	};

	// TODO: Do we need this?
	todo!()
}

async fn products(State(global): State<Arc<Global>>) -> Result<Json<Vec<types::Product>>, ApiError> {
	todo!()
}

#[derive(Debug, serde::Deserialize)]
struct RedeemRequest {
	code: String,
}

#[derive(Debug, serde::Serialize)]
struct RedeemResponse {
	/// Url that the website will open
	authorize_url: Option<String>,
    /// list of ids of cosmetics that the user received
    items: Vec<String>,
}

async fn redeem(
	State(global): State<Arc<Global>>,
	Json(body): Json<RedeemRequest>,
) -> Result<Json<RedeemResponse>, ApiError> {
	todo!()
}
