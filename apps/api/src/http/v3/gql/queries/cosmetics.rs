use async_graphql::{Context, Object};
use shared::{database::Id, types::old::{CosmeticBadgeModel, CosmeticPaintModel}};

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
    async fn cosmetics<'ctx>(&self, ctx: &Context<'ctx>, list: Vec<Id<()>>) -> Result<CosmeticsQueryResponse, ApiError> {
        Ok(CosmeticsQueryResponse::default())
    }
}
