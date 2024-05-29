use async_graphql::{ComplexObject, Context, Object, SimpleObject};
use shared::database::RoleId;

use crate::http::error::ApiError;

use super::users::User;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/roles.gql

#[derive(Default)]
pub struct RolesQuery;

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct Role {
    id: RoleId,
    name: String,
    color: i32,
    allowed: String,
    denied: String,
    position: u32,
    // created_at
    invisible: bool,
    // members
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl Role {
    async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.id.timestamp()
    }

    async fn members(&self, page: Option<u32>, limit: Option<u32>) -> Result<Vec<User>, ApiError> {
        Err(ApiError::NOT_IMPLEMENTED)
    }
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl RolesQuery {
    async fn roles<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<Role>, ApiError> {
        Err(ApiError::NOT_IMPLEMENTED)
    }

    async fn role<'ctx>(&self, ctx: &Context<'ctx>, id: RoleId) -> Result<Option<Role>, ApiError> {
        Err(ApiError::NOT_IMPLEMENTED)
    }
}
