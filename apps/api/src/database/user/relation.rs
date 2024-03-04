use postgres_types::{FromSql, ToSql};

use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserRelation {
	pub user_id: ulid::Ulid,
	pub other_user_id: ulid::Ulid,
	pub kind: UserRelationKind,
	pub created_at: chrono::DateTime<chrono::Utc>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: UserRelationData,
}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "user_relation_kind")]
pub enum UserRelationKind {
	#[default]
	#[postgres(name = "NOTHING")]
	Nothing,
	#[postgres(name = "FOLLOW")]
	Follow,
	#[postgres(name = "BLOCK")]
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