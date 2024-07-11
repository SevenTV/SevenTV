use derive_builder::Builder;
use mongodb::options::IndexOptions;

use self::permissions::Permissions;
use super::GenericCollection;
use crate::database::{Collection, Id};

pub mod permissions;

pub type RoleId = Id<Role>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct Role {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: RoleId,
	pub name: String,
	#[builder(default)]
	pub description: Option<String>,
	#[builder(default)]
	pub tags: Vec<String>,
	pub permissions: Permissions,
	#[builder(default = "false")]
	pub hoist: bool,
	#[builder(default)]
	pub color: Option<i32>,
	pub rank: i32,
}

impl Collection for Role {
	const COLLECTION_NAME: &'static str = "roles";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"rank": 1,
				})
				.options(IndexOptions::builder().unique(true).build())
				.build(),
		]
	}
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<Role>()]
}
