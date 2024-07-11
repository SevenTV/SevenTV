use derive_builder::Builder;

use super::user::UserId;
use super::GenericCollection;
use crate::database::{Collection, Id};

pub type AutomodRuleId = Id<AutomodRule>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct AutomodRule {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: AutomodRuleId,
	pub name: String,
	pub description: String,
	#[builder(default)]
	pub tags: Vec<String>,
	pub added_by: UserId,
	pub kind: AutomodRuleKind,
	#[builder(default = "true")]
	pub enabled: bool,
	#[builder(default)]
	pub words: Vec<String>,
	#[builder(default)]
	pub allowed_words: Vec<String>,
	#[builder(default)]
	pub action: Option<AutomodRuleAction>,
}

impl Collection for AutomodRule {
	const COLLECTION_NAME: &'static str = "automod_rules";
}

#[derive(Debug, serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Clone, Default)]
#[repr(u8)]
pub enum AutomodRuleKind {
	#[default]
	Normal = 0,
	Regex = 1,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
// Not sure what the difference between these two is
pub enum AutomodRuleAction {
	Timeout(i64),
	Ban(i64),
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<AutomodRule>()]
}
