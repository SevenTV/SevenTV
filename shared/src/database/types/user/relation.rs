use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct UserRelation {
	pub user_id: ulid::Ulid,
	pub other_user_id: ulid::Ulid,
	pub kind: UserRelationKind,
	pub created_at: chrono::DateTime<chrono::Utc>,
	pub data: UserRelationData,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum UserRelationKind {
	#[default]
	Nothing,
	Follow,
	Block,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct UserRelationData {
	pub notes: String,
	pub muted: Option<MutedState>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
pub enum MutedState {
	#[default]
	Permanent,
	Temporary(chrono::DateTime<chrono::Utc>),
}

impl Table for UserRelation {
	const TABLE_NAME: &'static str = "user_relations";
}
