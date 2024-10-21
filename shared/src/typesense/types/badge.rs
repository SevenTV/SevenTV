use chrono::Utc;

use super::{TypesenseCollection, TypesenseGenericCollection};
use crate::database::badge::BadgeId;
use crate::database::user::UserId;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "badges")]
#[serde(deny_unknown_fields)]
pub struct Badge {
	pub id: BadgeId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub created_by_id: UserId,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<crate::database::badge::Badge> for Badge {
	fn from(badge: crate::database::badge::Badge) -> Self {
		Self {
			id: badge.id,
			name: badge.name,
			description: badge.description,
			tags: badge.tags,
			created_by_id: badge.created_by_id,
			created_at: badge.id.timestamp().timestamp_millis(),
			updated_at: badge.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<Badge>()]
}
