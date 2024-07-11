use derive_builder::Builder;

use super::automod::AutomodRuleId;
use super::emote_set::EmoteSetId;
use super::GenericCollection;
use crate::database::Collection;

pub type GlobalConfigId = ();

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct GlobalConfig {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: GlobalConfigId,
	#[builder(default)]
	pub alerts: GlobalConfigAlerts,
	pub emote_set_id: EmoteSetId,
	#[builder(default)]
	pub automod_rule_ids: Vec<AutomodRuleId>,
	pub normal_emote_set_slot_capacity: i32,
	pub personal_emote_set_slot_capacity: i32,
}

impl Collection for GlobalConfig {
	const COLLECTION_NAME: &'static str = "global_config";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default, Builder)]
#[serde(deny_unknown_fields)]
pub struct GlobalConfigAlerts {
	#[builder(default)]
	pub message: Option<String>,
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<GlobalConfig>()]
}
