use crate::database;
use crate::database::user::relation::{MutedState, UserRelationId, UserRelationKind};
use crate::database::user::UserId;
use crate::typesense::types::{TypesenseCollection, TypesenseGenericCollection, TypesenseString};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "user_relations")]
#[serde(deny_unknown_fields)]
pub struct UserRelation {
	pub id: TypesenseString<UserRelationId>,
	pub user_id: UserId,
	pub other_user_id: UserId,
	pub kind: UserRelationKind,
	pub notes: String,
	pub muted: bool,
	pub muted_expiry: Option<i64>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::user::relation::UserRelation> for UserRelation {
	fn from(value: database::user::relation::UserRelation) -> Self {
		Self {
			id: value.id.into(),
			user_id: value.id.user_id,
			other_user_id: value.id.other_user_id,
			kind: value.kind,
			notes: value.notes,
			muted: value.muted.is_some(),
			muted_expiry: value.muted.and_then(|muted| match muted {
				MutedState::Temporary(expiry) => Some(expiry.timestamp_millis()),
				MutedState::Permanent => None,
			}),
			created_at: value.created_at.timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: value.updated_at.timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<UserRelation>()]
}
