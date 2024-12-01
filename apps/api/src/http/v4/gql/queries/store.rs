use std::sync::Arc;

use async_graphql::Context;
use shared::database::entitlement::EntitlementEdgeKind;
use shared::database::entitlement_edge::EntitlementEdgeGraphTraverse;
use shared::database::graph::{Direction, GraphTraverse};
use shared::database::paint::PaintId;
use shared::database::product::SubscriptionBenefitCondition;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v4::gql::types::Paint;

#[derive(Default)]
pub struct StoreQuery;

#[async_graphql::Object]
impl StoreQuery {
	#[tracing::instrument(skip_all, name = "StoreQuery::monthly_paints")]
	async fn monthly_paints(&self, ctx: &Context<'_>) -> Result<Vec<Paint>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let products = global
			.subscription_products_loader
			.load(())
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription products"))?
			.unwrap_or_default();

		// find relevant benefits
		let months_benefit_ids: Vec<_> = products
			.into_iter()
			.flat_map(|p| p.benefits)
			.filter(|b| match &b.condition {
				SubscriptionBenefitCondition::TimePeriod(tp) => {
					tp.start <= chrono::Utc::now() && tp.end > chrono::Utc::now()
				}
				_ => false,
			})
			.map(|b| EntitlementEdgeKind::SubscriptionBenefit {
				subscription_benefit_id: b.id,
			})
			.collect();

		let traverse = &EntitlementEdgeGraphTraverse {
			inbound_loader: &global.entitlement_edge_inbound_loader,
			outbound_loader: &global.entitlement_edge_outbound_loader,
		};

		// follow the graph
		let edges = traverse
			.traversal(Direction::Outbound, months_benefit_ids)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to traverse entitlement edges"))?;

		// find the paints
		let current_paints_ids: Vec<PaintId> = edges
			.into_iter()
			.filter_map(|e| match e.id.to {
				EntitlementEdgeKind::Paint { paint_id } => Some(paint_id),
				_ => None,
			})
			.collect();

		let current_paints = global
			.paint_by_id_loader
			.load_many(current_paints_ids)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load paints"))?
			.into_values()
			.map(|p| Paint::from_db(p, &global.config.api.cdn_origin))
			.collect();

		Ok(current_paints)
	}
}
