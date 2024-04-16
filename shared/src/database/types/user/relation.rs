use bson::oid::ObjectId;

use crate::database::Collection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserRelation {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub user_id: ObjectId,
	pub other_user_id: ObjectId,
	pub kind: UserRelationKind,
	pub created_at: chrono::DateTime<chrono::Utc>,
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
	const NAME: &'static str = "user_relations";
}
