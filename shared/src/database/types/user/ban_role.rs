use super::{Collection, Permissions};
use crate::database::Id;

pub type UserBanRoleId = Id<UserBanRole>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserBanRole {
	#[serde(rename = "_id")]
	pub id: UserBanRoleId,
	pub name: String,
	pub description: Option<String>,
	pub permissions: Permissions,
	pub black_hole: bool,
}

impl Collection for UserBanRole {
	const COLLECTION_NAME: &'static str = "user_ban_roles";
}
