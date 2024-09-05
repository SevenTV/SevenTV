use chrono::Utc;

use crate::database;
use crate::database::user::ban_template::UserBanTemplateId;
use crate::database::user::UserId;
use crate::typesense::types::duration_unit::DurationUnit;
use crate::typesense::types::{TypesenseCollection, TypesenseGenericCollection};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "user_ban_templates")]
#[serde(deny_unknown_fields)]
pub struct UserBanTemplate {
	pub id: UserBanTemplateId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub pinned: bool,
	pub created_by: UserId,
	pub duration: Option<DurationUnit>,
	pub duration_value: Option<i32>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::user::ban_template::UserBanTemplate> for UserBanTemplate {
	fn from(value: database::user::ban_template::UserBanTemplate) -> Self {
		let (duration, duration_value) = value
			.duration
			.map(DurationUnit::split)
			.map(|(duration, value)| (Some(duration), Some(value)))
			.unwrap_or((None, None));

		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			pinned: value.pinned,
			created_by: value.created_by,
			duration,
			duration_value,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<UserBanTemplate>()]
}
