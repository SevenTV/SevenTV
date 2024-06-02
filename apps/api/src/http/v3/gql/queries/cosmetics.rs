use std::sync::Arc;

use async_graphql::{Context, Object};
use futures::StreamExt;
use hyper::StatusCode;
use mongodb::bson::doc;
use shared::database::{Badge, Collection, Paint};
use shared::old_types::{CosmeticBadgeModel, CosmeticPaintModel};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::gql::object_id::ObjectId;

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
		list: Option<Vec<ObjectId<()>>>,
	) -> Result<CosmeticsQueryResponse, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let list = list.unwrap_or_default();

		if list.len() > 1000 {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "list too large"));
		}

		let (paints, badges) = if list.is_empty() {
			let paints = Paint::collection(global.db())
				.find(doc! {}, None)
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.filter_map(|p| async { CosmeticPaintModel::from_db(p.ok()?, &global.config().api.cdn_base_url) })
				.collect::<Vec<_>>()
				.await;

			let badges = Badge::collection(global.db())
				.find(doc! {}, None)
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.filter_map(|b| async { CosmeticBadgeModel::from_db(b.ok()?, &global.config().api.cdn_base_url) })
				.collect::<Vec<_>>()
				.await;

			(paints, badges)
		} else {
			let paints = global
				.paint_by_id_loader()
				.load_many(list.clone().into_iter().map(|id| id.id().cast()))
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.into_values()
				.filter_map(|p| CosmeticPaintModel::from_db(p, &global.config().api.cdn_base_url))
				.collect();

			let badges = global
				.badge_by_id_loader()
				.load_many(list.into_iter().map(|id| id.id().cast()))
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.into_values()
				.filter_map(|b| CosmeticBadgeModel::from_db(b, &global.config().api.cdn_base_url))
				.collect();

			(paints, badges)
		};

		Ok(CosmeticsQueryResponse { paints, badges })
	}
}
