use async_graphql::SimpleObject;
use shared::database::user::{profile_picture::UserProfilePictureId, UserId};

use super::Image;

#[derive(Debug, Clone, SimpleObject)]
pub struct UserProfilePicture {
	pub id: UserProfilePictureId,
	pub user_id: UserId,
	pub images: Vec<Image>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserProfilePicture {
	pub fn from_db(value: shared::database::user::profile_picture::UserProfilePicture, cdn_base_url: &url::Url) -> Self {
		Self {
			id: value.id,
			user_id: value.user_id,
			images: value
				.image_set
				.outputs
				.into_iter()
				.map(|o| Image::from_db(o, cdn_base_url))
				.collect(),
			updated_at: value.updated_at,
		}
	}
}
