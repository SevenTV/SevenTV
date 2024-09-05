use super::user::UserId;
use super::MongoGenericCollection;
use crate::database::{Id, MongoCollection};
use crate::typesense::types::impl_typesense_type;

pub type AutomodRuleId = Id<AutomodRule>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "automod_rules")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[mongo(search = "crate::typesense::types::automod::AutomodRule")]
#[serde(deny_unknown_fields)]
pub struct AutomodRule {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: AutomodRuleId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub created_by: UserId,
	pub enabled: bool,
	pub blacklisted_phrases: Vec<AutomodRulePhrase>,
	pub whitelisted_phrases: Vec<AutomodRulePhrase>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct AutomodRulePhrase {
	pub phrase: String,
	pub kind: AutomodRuleKind,
}

#[derive(
	Debug, serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd,
)]
#[repr(u8)]
pub enum AutomodRuleKind {
	#[default]
	Normal = 0,
	Regex = 1,
}

impl_typesense_type!(AutomodRuleKind, Int32);

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<AutomodRule>()]
}
