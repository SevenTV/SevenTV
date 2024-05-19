use std::sync::Arc;

use axum::response::IntoResponse;
// use mongodb::options::UpdateOptions;
use shared::database::Collection;

use crate::global::Global;
use crate::http::error::ApiError;

#[tracing::instrument(skip(global), fields(subscription_id = %subscription.id))]
pub async fn created(global: Arc<Global>, subscription: stripe::Subscription) -> Result<impl IntoResponse, ApiError> {
	handle(&global, subscription).await
}

#[tracing::instrument(skip(global), fields(subscription_id = %subscription.id))]
pub async fn updated(global: Arc<Global>, subscription: stripe::Subscription) -> Result<impl IntoResponse, ApiError> {
	handle(&global, subscription).await
}

#[tracing::instrument(skip(global), fields(subscription_id = %subscription.id))]
pub async fn deleted(global: Arc<Global>, subscription: stripe::Subscription) -> Result<impl IntoResponse, ApiError> {
	handle(&global, subscription).await
}

async fn handle(global: &Arc<Global>, subscription: stripe::Subscription) -> Result<impl IntoResponse, ApiError> {
	let collection = shared::database::Subscription::collection(global.db());

	// let item = subscription.items.data.first()

	// collection.update_one(
	//     bson::doc! {
	//         "_id": subscription.id.as_str()
	//     },
	//     bson::doc! {
	//         "$set": {
	//             ""
	//         },
	//         "$setOnInsert": {
	//             "product_id":
	//         }
	//     },
	//     Some(
	//         UpdateOptions::builder()
	//             .upsert(true)
	//     )
	// ).await.map_err(|err| {
	//     tracing::error!("failed to update subscription: {err}");
	//     ApiError::INTERNAL_SERVER_ERROR
	// })?;

	Ok(())
}
