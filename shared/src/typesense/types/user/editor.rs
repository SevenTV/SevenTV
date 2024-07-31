use crate::database;
use crate::database::user::editor::{UserEditorId, UserEditorState};
use crate::database::user::UserId;
use crate::typesense::types::{TypesenseCollection, TypesenseGenericCollection, TypesenseString};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "user_editors")]
#[serde(deny_unknown_fields)]
pub struct UserEditor {
	pub id: TypesenseString<UserEditorId>,
	pub user_id: UserId,
	pub editor_id: UserId,
	pub state: UserEditorState,
	pub notes: Option<String>,
	pub added_by_id: UserId,
	#[typesense(default_sort)]
	pub added_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::user::editor::UserEditor> for UserEditor {
	fn from(value: database::user::editor::UserEditor) -> Self {
		Self {
			id: value.id.into(),
			user_id: value.id.user_id,
			editor_id: value.id.editor_id,
			state: value.state,
			notes: value.notes,
			added_by_id: value.added_by_id,
			added_at: value.added_at.timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: value.updated_at.timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<UserEditor>()]
}
