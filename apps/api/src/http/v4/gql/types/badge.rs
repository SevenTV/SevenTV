use shared::database::badge::BadgeId;
use shared::database::user::UserId;

use super::Image;

#[derive(async_graphql::SimpleObject)]
pub struct Badge {
	pub id: BadgeId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub images: Vec<Image>,
	pub created_by_id: UserId,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Badge {
	pub fn from_db(value: shared::database::badge::Badge, cdn_base_url: &url::Url) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			images: value
				.image_set
				.outputs
				.into_iter()
				.map(|o| Image::from_db(o, cdn_base_url))
				.collect(),
			created_by_id: value.created_by_id,
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		}
	}
}
