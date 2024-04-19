use super::UserId;
use crate::database::{Collection, Id};

pub type UserRelationId = Id<UserRelation>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserRelation {
	#[serde(rename = "_id")]
	pub id: UserRelationId,
	pub user_id: UserId,
	pub other_user_id: UserId,
	pub kind: UserRelationKind,
	pub notes: String,
	pub muted: Option<MutedState>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum UserRelationKind {
	#[default]
	Nothing,
	Follow,
	Block,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum MutedState {
	#[default]
	Permanent,
	Temporary(chrono::DateTime<chrono::Utc>),
}

impl Collection for UserRelation {
	const COLLECTION_NAME: &'static str = "user_relations";
}
