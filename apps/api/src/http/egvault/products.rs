use std::sync::Arc;

use axum::extract::State;
use axum::{Extension, Json};
use futures::TryStreamExt;
use shared::database::entitlement::EntitlementEdgeKind;
use shared::database::product::{SubscriptionBenefitCondition, SubscriptionProduct};
use shared::database::queries::filter;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::rest::types::{self, Plan};

pub async fn products(
	State(global): State<Arc<Global>>,
	Extension(ip): Extension<std::net::IpAddr>,
) -> Result<Json<Vec<types::Product>>, ApiError> {
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

	let currency = if let Some(country_code) = global.geoip().and_then(|g| g.lookup(ip)).and_then(|c| c.iso_code) {
		let global = global
			.global_config_loader
			.load(())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		global.country_currency_overrides.get(country_code).copied()
	} else {
		None
	};

	let plans = products
		.iter()
		.cloned()
		.flat_map(|p| {
			p.variants.into_iter().filter_map(move |v| {
				if v.active {
					Plan::from_variant(v, currency, p.default_currency)
				} else {
					None
				}
			})
		})
		.collect();

	// find relevant benefits
	let months_benefit_ids: Vec<_> = products
		.into_iter()
		.flat_map(|p| p.benefits)
		.filter(|b| match &b.condition {
			SubscriptionBenefitCondition::TimePeriod(tp) => tp.start <= chrono::Utc::now() && tp.end > chrono::Utc::now(),
			_ => false,
		})
		.map(|b| EntitlementEdgeKind::SubscriptionBenefit {
			subscription_benefit_id: b.id,
		})
		.collect();

	// follow the graph
	let edges = global
		.entitlement_edge_outbound_loader
		.load_many(months_benefit_ids)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

	// find the paints
	let current_paints = edges
		.into_values()
		.flatten()
		.filter_map(|e| match e.id.to {
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
