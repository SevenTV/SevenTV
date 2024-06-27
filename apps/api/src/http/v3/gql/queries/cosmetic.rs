use std::sync::Arc;

use async_graphql::{Context, Object};
use hyper::StatusCode;
use mongodb::bson::doc;
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
		list: Option<Vec<GqlObjectId>>,
	) -> Result<CosmeticsQueryResponse, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let list = list.unwrap_or_default();

		if list.len() > 1000 {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "list too large"));
		}

		let paints = global
			.paint_by_id_loader()
			.load_many(list.clone().into_iter().map(|id| id.id()))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.into_values()
			.filter_map(|p| CosmeticPaintModel::from_db(p, &global.config().api.cdn_origin))
			.collect();

		let badges = global
			.badge_by_id_loader()
			.load_many(list.into_iter().map(|id| id.id()))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.into_values()
			.filter_map(|b| CosmeticBadgeModel::from_db(b, &global.config().api.cdn_origin))
			.collect();

		Ok(CosmeticsQueryResponse { paints, badges })
	}
}
