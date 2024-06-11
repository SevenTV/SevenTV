use async_graphql::{InputObject, Object};
use shared::database::RolePermission;
use shared::old_types::RoleObjectId;

use crate::http::error::ApiError;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::Role;

#[derive(Default)]
pub struct RolesMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl RolesMutation {
	#[graphql(guard = "PermissionGuard::one(RolePermission::Create)")]
	async fn create_role(&self, data: CreateRoleInput) -> Result<Role, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Edit)")]
	async fn edit_role(&self, role_id: RoleObjectId, data: EditRoleInput) -> Result<Role, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Delete)")]
	async fn delete_role(&self, role_id: RoleObjectId) -> Result<String, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CreateRoleInput {
	name: String,
	color: u32,
	allowed: String,
	denied: String,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EditRoleInput {
	name: Option<String>,
	color: Option<u32>,
	allowed: Option<String>,
	denied: Option<String>,
	position: Option<u32>,
}
