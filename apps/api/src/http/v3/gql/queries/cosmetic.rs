use std::future::IntoFuture;
use std::sync::Arc;

use async_graphql::{Context, Object};
use futures::{TryFutureExt, TryStreamExt};
use mongodb::bson::doc;
use shared::database::badge::{Badge, BadgeId};
use shared::database::paint::{Paint, PaintId};
use shared::database::queries::filter;
use shared::database::{Id, MongoCollection};
use shared::old_types::cosmetic::{CosmeticBadgeModel, CosmeticPaintModel};
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::ApiError;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/cosmetics.gql

#[derive(Default)]
pub struct CosmeticsQuery;

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(name = "CosmeticsQuery", rename_fields = "snake_case")]
pub struct CosmeticsQueryResponse {
	paints: Vec<CosmeticPaintModel>,
	badges: Vec<CosmeticBadgeModel>,
}

#[Object(name = "CosmeticsRootQuery", rename_fields = "camelCase", rename_args = "snake_case")]
impl CosmeticsQuery {
	async fn cosmetics<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(max_items = 100))] list: Option<Vec<GqlObjectId>>,
	) -> Result<CosmeticsQueryResponse, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let list = list.unwrap_or_default();

		if list.is_empty() {
			// return all cosmetics when empty list is provided

			let paints = Paint::collection(&global.db)
				.find(filter::filter!(Paint {}))
				.into_future()
				.and_then(|f| f.try_collect::<Vec<Paint>>())
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to query paints");
					ApiError::INTERNAL_SERVER_ERROR
				})?
				.into_iter()
				.filter_map(|p| CosmeticPaintModel::from_db(p, &global.config.api.cdn_origin))
				.collect();

			let badges = Badge::collection(&global.db)
				.find(filter::filter!(Badge {}))
				.into_future()
				.and_then(|f| f.try_collect::<Vec<Badge>>())
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to query badges");
					ApiError::INTERNAL_SERVER_ERROR
				})?
				.into_iter()
				.filter_map(|b: Badge| CosmeticBadgeModel::from_db(b, &global.config.api.cdn_origin))
				.collect();

			Ok(CosmeticsQueryResponse { paints, badges })
		} else {
			let list: Vec<Id<()>> = list.clone().into_iter().map(|id| id.id()).collect();

			let ids: Vec<PaintId> = list.iter().cloned().map(|id| id.cast()).collect();

			let paints = Paint::collection(&global.db)
				.find(filter::filter!(Paint {
					#[query(rename = "_id", selector = "in")]
					id: ids,
				}))
				.sort(doc! { "_id": 1 })
				.into_future()
				.and_then(|f| f.try_collect::<Vec<Paint>>())
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to query paints");
					ApiError::INTERNAL_SERVER_ERROR
				})?
				.into_iter()
				.filter_map(|p| CosmeticPaintModel::from_db(p, &global.config.api.cdn_origin))
				.collect();

			let ids: Vec<BadgeId> = list.into_iter().map(|id| id.cast()).collect();

			let badges = Badge::collection(&global.db)
				.find(filter::filter!(Badge {
					#[query(rename = "_id", selector = "in")]
					id: ids,
				}))
				.sort(doc! { "_id": 1 })
				.into_future()
				.and_then(|f| f.try_collect::<Vec<Badge>>())
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to query badges");
					ApiError::INTERNAL_SERVER_ERROR
				})?
				.into_iter()
				.filter_map(|b: Badge| CosmeticBadgeModel::from_db(b, &global.config.api.cdn_origin))
				.collect();

			Ok(CosmeticsQueryResponse { paints, badges })
		}
	}
}
