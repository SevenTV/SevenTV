use std::collections::HashMap;

use super::automod::AutomodRuleId;
use super::emote_set::EmoteSetId;
use super::MongoGenericCollection;
use crate::database::MongoCollection;

pub type GlobalConfigId = ();

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "global_config")]
#[serde(deny_unknown_fields, default)]
pub struct GlobalConfig {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: GlobalConfigId,
	pub alerts: GlobalConfigAlerts,
	pub emote_set_id: EmoteSetId,
	pub automod_rule_ids: Vec<AutomodRuleId>,
	pub trending_emote_count: usize,
	pub country_currency_overrides: HashMap<String, stripe::Currency>,
}

impl Default for GlobalConfig {
	fn default() -> Self {
		Self {
			id: (),
			alerts: Default::default(),
			emote_set_id: Default::default(),
			automod_rule_ids: Default::default(),
			trending_emote_count: 500,
			country_currency_overrides: Default::default(),
		}
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct GlobalConfigAlerts {
	pub message: Option<String>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<GlobalConfig>()]
}
