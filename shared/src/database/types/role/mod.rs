use self::permissions::Permissions;
use super::GenericCollection;
use crate::database::{Collection, Id};

pub mod permissions;

pub type RoleId = Id<Role>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Role {
	#[serde(rename = "_id")]
	pub id: RoleId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub permissions: Permissions,
	pub hoist: bool,
	pub color: Option<u32>,
}

impl Collection for Role {
	const COLLECTION_NAME: &'static str = "roles";
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<Role>()]
}
