use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct AutomodRule {
	pub id: ulid::Ulid,
	pub name: String,
	pub description: String,
	pub tags: Vec<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub priority: i16,
	pub added_by: Option<ulid::Ulid>,
	pub data: AutomodRuleData,
}

impl Table for AutomodRule {
	const TABLE_NAME: &'static str = "automod_rules";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct AutomodRuleData {
	pub kind: AutomodRuleKind,
	pub enabled: bool,
	pub words: Vec<String>,
	pub allowed_words: Vec<String>,
	pub action: Option<AutomodRuleAction>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
pub enum AutomodRuleKind {
	#[default]
	Normal,
	Regex,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum AutomodRuleAction {
	Timeout(std::time::Duration),
	Ban(std::time::Duration),
}
