use crate::database;
use crate::database::user::ban::UserBanId;
use crate::database::user::ban_template::UserBanTemplateId;
use crate::database::user::UserId;
use crate::typesense::types::{TypesenseCollection, TypesenseGenericCollection};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "user_bans")]
#[serde(deny_unknown_fields)]
pub struct UserBan {
	pub id: UserBanId,
	pub user_id: UserId,
	pub created_by_id: UserId,
	pub reason: String,
	pub tags: Vec<String>,
	pub expires_at: Option<i64>,
	pub removed_at: Option<i64>,
	pub removed_by_id: Option<UserId>,
	pub template_id: Option<UserBanTemplateId>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::user::ban::UserBan> for UserBan {
	fn from(value: database::user::ban::UserBan) -> Self {
		Self {
			id: value.id,
			user_id: value.user_id,
			created_by_id: value.created_by_id,
			reason: value.reason,
			tags: value.tags,
			expires_at: value.expires_at.map(|date| date.timestamp_millis()),
			removed_at: value.removed.as_ref().map(|removed| removed.removed_at.timestamp_millis()),
			removed_by_id: value.removed.as_ref().map(|removed| removed.removed_by_id),
			template_id: value.template_id,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: value.updated_at.timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<UserBan>()]
}
