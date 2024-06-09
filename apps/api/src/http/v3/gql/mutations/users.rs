use async_graphql::{ComplexObject, InputObject, Object, SimpleObject};
use shared::database::{Permission, RolePermission, UserPermission};
use shared::old_types::{CosmeticKind, EmoteSetObjectId, ObjectId, RoleObjectId, UserObjectId};

use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::{
	error::ApiError,
	v3::gql::queries::{UserConnection, UserEditor},
};

use super::emote_sets::ListItemAction;

#[derive(Default)]
pub struct UsersMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl UsersMutation {
	async fn user(&self, id: UserObjectId) -> UserOps {
		UserOps { id }
	}
}

#[derive(SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct UserOps {
	id: UserObjectId,
}

#[ComplexObject(rename_fields = "camelCase", rename_args = "snake_case")]
impl UserOps {
	#[graphql(guard = "PermissionGuard::one(UserPermission::Edit)")]
	async fn connections(
		&self,
		id: String,
		data: UserConnectionUpdate,
	) -> Result<Option<Vec<Option<UserConnection>>>, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	#[graphql(guard = "PermissionGuard::one(UserPermission::Edit)")]
	async fn editors(
		&self,
		editor_id: UserObjectId,
		data: UserEditorUpdate,
	) -> Result<Option<Vec<Option<UserEditor>>>, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	#[graphql(guard = "PermissionGuard::one(UserPermission::Edit)")]
	async fn cosmetics(&self, update: UserCosmeticUpdate) -> Result<bool, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	#[graphql(
		guard = "PermissionGuard::all([Permission::from(RolePermission::Assign), Permission::from(UserPermission::Edit)])"
	)]
	async fn roles(&self, action: ListItemAction) -> Result<RoleObjectId, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UserConnectionUpdate {
	emote_set_id: Option<EmoteSetObjectId>,
	unlink: Option<bool>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UserEditorUpdate {
	permissions: Option<u32>,
	visible: Option<bool>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UserCosmeticUpdate {
	id: ObjectId<()>,
	kind: CosmeticKind,
	selected: bool,
}
