use bson::oid::ObjectId;

use crate::database::Collection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct GlobalConfig {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub alerts: GlobalConfigAlerts,
	pub emote_set_ids: Vec<ObjectId>,
	pub role_ids: Vec<ObjectId>,
}

impl Collection for GlobalConfig {
	const COLLECTION_NAME: &'static str = "global_config";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct GlobalConfigAlerts {}
