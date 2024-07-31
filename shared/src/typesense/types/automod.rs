use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::{TypesenseCollection, TypesenseGenericCollection};
use crate::database::automod::{AutomodRuleId, AutomodRuleKind};
use crate::database::user::UserId;
use crate::database::{self};

#[derive(Debug, Serialize, Deserialize, Clone, TypesenseCollection)]
#[typesense(collection_name = "automod_rules")]
#[serde(deny_unknown_fields)]
pub struct AutomodRule {
	pub id: AutomodRuleId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub added_by: UserId,
	pub enabled: bool,
	pub contains_regex: bool,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::automod::AutomodRule> for AutomodRule {
	fn from(rule: database::automod::AutomodRule) -> Self {
		Self {
			id: rule.id,
			name: rule.name,
			description: rule.description,
			tags: rule.tags,
			added_by: rule.created_by,
			contains_regex: rule
				.blacklisted_phrases
				.iter()
				.any(|phrase| phrase.kind == AutomodRuleKind::Regex),
			enabled: rule.enabled,
			created_at: rule.id.timestamp().timestamp_millis(),
			updated_at: rule.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<AutomodRule>()]
}
