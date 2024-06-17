use super::UserId;
use crate::database::types::GenericCollection;
use crate::database::Collection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct UserRelationId {
	pub user_id: UserId,
	pub other_user_id: UserId,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserRelation {
	#[serde(rename = "_id")]
	pub id: UserRelationId,
	pub kind: UserRelationKind,
	pub notes: String,
	pub muted: Option<MutedState>,
}

#[derive(Debug, Clone, Default, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum UserRelationKind {
	#[default]
	Nothing = 0,
	Follow = 1,
	Block = 2,
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

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"_id.user_id": 1,
					"_id.other_user_id": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"_id.other_user_id": 1,
					"_id.user_id": 1,
				})
				.build(),
		]
	}
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<UserRelation>()]
}
