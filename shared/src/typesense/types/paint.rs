use super::TypesenseGenericCollection;
use crate::database::paint::PaintId;
use crate::database::user::UserId;
use crate::typesense::types::TypesenseCollection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "paints")]
#[serde(deny_unknown_fields)]
pub struct Paint {
	pub id: PaintId,
	pub name: String,
	pub description: String,
	pub tags: Vec<String>,
	pub created_by: UserId,
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<crate::database::paint::Paint> for Paint {
	fn from(value: crate::database::paint::Paint) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			created_by: value.created_by,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: chrono::Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<Paint>()]
}
