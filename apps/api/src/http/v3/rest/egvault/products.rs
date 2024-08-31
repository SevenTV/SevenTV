use std::sync::Arc;

use axum::{extract::State, Json};
use futures::TryStreamExt;
use shared::database::{
	entitlement::EntitlementEdgeKind,
	product::{SubscriptionBenefitCondition, SubscriptionProduct},
	queries::filter,
    MongoCollection,
};

use crate::{
	global::Global,
	http::{error::ApiError, v3::rest::types},
};

pub async fn products(State(global): State<Arc<Global>>) -> Result<Json<Vec<types::Product>>, ApiError> {
	let products: Vec<SubscriptionProduct> = SubscriptionProduct::collection(&global.db)
		.find(filter::filter! {
			SubscriptionProduct {}
		})
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to query subscription products");
			ApiError::INTERNAL_SERVER_ERROR
		})?
		.try_collect()
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to collect subscription products");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

	let plans = products.iter().cloned().map(Into::into).collect();

	let current_paints = products
		.into_iter()
		.flat_map(|p| p.benefits)
		.filter(|b| match &b.condition {
			SubscriptionBenefitCondition::TimePeriod(time_period) => {
				time_period.start <= chrono::Utc::now() && time_period.end > chrono::Utc::now()
			}
			_ => false,
		})
		.filter_map(|b| match b.entitlement {
			EntitlementEdgeKind::Paint { paint_id } => Some(paint_id),
			_ => None,
		})
		.collect();

	Ok(Json(vec![types::Product {
		name: "subscription".to_string(),
		plans,
		current_paints,
	}]))
}
