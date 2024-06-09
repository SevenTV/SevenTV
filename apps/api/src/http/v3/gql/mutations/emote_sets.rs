use async_graphql::{ComplexObject, Enum, InputObject, Object, SimpleObject};
use shared::old_types::{EmoteSetObjectId, UserObjectId};
use shared::database::EmoteSetPermission;

use crate::http::error::ApiError;
use crate::http::v3::gql::queries::{ActiveEmote, EmoteSet};
use crate::http::v3::gql::guards::PermissionGuard;

#[derive(Default)]
pub struct EmoteSetsMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmoteSetsMutation {
    async fn emote_set(&self, id: EmoteSetObjectId) -> Option<EmoteSetOps> {
        Some(EmoteSetOps { id })
    }

    #[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Create)")]
    async fn create_emote_set(&self, user_id: UserObjectId, data: CreateEmoteSetInput) -> Result<EmoteSet, ApiError> {
        Err(ApiError::NOT_IMPLEMENTED)
    }
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CreateEmoteSetInput {
    name: String,
    privileged: Option<bool>,
}

#[derive(SimpleObject, Default)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmoteSetOps {
    id: EmoteSetObjectId,
}

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum ListItemAction {
    Add,
    Update,
    Remove,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UpdateEmoteSetInput {
    name: Option<String>,
    capacity: Option<u32>,
    origins: Option<Vec<EmoteSetOriginInput>>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EmoteSetOriginInput {
    id: EmoteSetObjectId,
    weight: Option<u32>,
    slices: Option<Vec<u32>>,
}

#[ComplexObject(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmoteSetOps {
    #[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Edit)")]
    async fn emotes(&self, id: EmoteSetObjectId, action: ListItemAction, name: Option<String>) -> Result<Vec<ActiveEmote>, ApiError> {
        Err(ApiError::NOT_IMPLEMENTED)
    }

    #[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Edit)")]
    async fn update(&self, data: UpdateEmoteSetInput) -> Result<EmoteSet, ApiError> {
        Err(ApiError::NOT_IMPLEMENTED)
    }

    #[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Delete)")]
    async fn delete(&self) -> Result<bool, ApiError> {
        Err(ApiError::NOT_IMPLEMENTED)
    }
}
