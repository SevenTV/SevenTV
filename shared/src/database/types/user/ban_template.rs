use derive_builder::Builder;

use super::Collection;
use crate::database::duration::DurationUnit;
use crate::database::role::permissions::Permissions;
use crate::database::types::GenericCollection;
use crate::database::Id;

pub type UserBanTemplateId = Id<UserBanTemplate>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct UserBanTemplate {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: UserBanTemplateId,
	pub name: String,
	#[builder(default)]
	pub description: Option<String>,
	#[builder(default)]
	pub black_hole: bool,
	pub permissions: Permissions,
	#[builder(default)]
	pub duration: Option<DurationUnit>,
}

impl Collection for UserBanTemplate {
	const COLLECTION_NAME: &'static str = "user_ban_templates";
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<UserBanTemplate>()]
}
