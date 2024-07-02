use super::UserId;
use crate::database::types::MongoGenericCollection;
use crate::database::MongoCollection;
use crate::typesense::types::impl_typesense_type;

#[derive(Debug, Clone, Copy, Default, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct UserRelationId {
	pub user_id: UserId,
	pub other_user_id: UserId,
}

impl std::fmt::Display for UserRelationId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:{}", self.user_id, self.other_user_id)
	}
}

impl std::str::FromStr for UserRelationId {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let parts = s.split(':').collect::<Vec<_>>();
		if parts.len() != 2 {
			return Err("invalid user relation id");
		}

		Ok(Self {
			user_id: parts[0].parse().map_err(|_| "invalid user id")?,
			other_user_id: parts[1].parse().map_err(|_| "invalid other user id")?,
		})
	}
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "user_relations")]
#[mongo(index(fields("_id.user_id" = 1, "_id.other_user_id" = 1)))]
#[mongo(index(fields("_id.other_user_id" = 1, "_id.user_id" = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct UserRelation {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: UserRelationId,
	pub kind: UserRelationKind,
	pub notes: String,
	pub muted: Option<MutedState>,
	#[serde(with = "crate::database::serde")]
	pub created_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum UserRelationKind {
	#[default]
	Nothing = 0,
	Follow = 1,
	Block = 2,
}

impl_typesense_type!(UserRelationKind, Int32);

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum MutedState {
	#[default]
	Permanent,
	Temporary(#[serde(with = "crate::database::serde")] chrono::DateTime<chrono::Utc>),
}

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<UserRelation>()]
}
