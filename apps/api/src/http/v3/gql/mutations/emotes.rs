use async_graphql::{ComplexObject, InputObject, Object, SimpleObject};
use shared::old_types::{EmoteObjectId, UserObjectId};
use shared::database::EmotePermission;

use crate::http::{error::ApiError, v3::gql::queries::Emote};
use crate::http::v3::gql::guards::PermissionGuard;

#[derive(Default)]
pub struct EmotesMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmotesMutation {
    async fn emote(&self, id: EmoteObjectId) -> EmoteOps {
        EmoteOps { id }
    }
}

#[derive(SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmoteOps {
    id: EmoteObjectId,
}

#[ComplexObject(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmoteOps {
    #[graphql(guard = "PermissionGuard::one(EmotePermission::Edit)")]
    async fn update(&self, params: EmoteUpdate, reason: Option<String>) -> Result<Emote, ApiError> {
        Err(ApiError::NOT_IMPLEMENTED)
    }

    #[graphql(guard = "PermissionGuard::one(EmotePermission::Admin)")]
    async fn merge(&self, target_id: EmoteObjectId, reason: Option<String>) -> Result<Emote, ApiError> {
        Err(ApiError::NOT_IMPLEMENTED)
    }

    #[graphql(guard = "PermissionGuard::one(EmotePermission::Admin)")]
    async fn rerun(&self) -> Result<Option<Emote>, ApiError> {
        Err(ApiError::NOT_IMPLEMENTED)
    }
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EmoteUpdate {
    name: Option<String>,
    version_name: Option<String>,
    version_description: Option<String>,
    flags: Option<u32>,
    owner_id: Option<UserObjectId>,
    tags: Option<Vec<String>>,
    listed: Option<bool>,
    personal_use: Option<bool>,
    deleted: Option<bool>,
}
