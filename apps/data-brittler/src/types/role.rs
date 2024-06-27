use mongodb::bson::oid::ObjectId;
use shared::database;
use shared::old_types::role_permission::RolePermission;

// https://github.com/SevenTV/Common/blob/master/structures/v3/type.role.go

#[derive(Debug, serde::Deserialize)]
pub struct Role {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub name: String,
	pub position: u32,
	pub color: i32,
	#[serde(default)]
	pub allowed: RolePermission,
	#[serde(default)]
	pub denied: RolePermission,
	pub discord_id: Option<u64>,
}

impl Role {
	pub fn to_new_permissions(&self) -> database::role::permissions::Permissions {
		RolePermission::to_new_permissions(self.allowed, self.denied)
	}
}
