use super::UserId;
use crate::database::image_set::ImageSet;
use crate::database::types::{MongoCollection, MongoGenericCollection};
use crate::database::Id;

pub type UserProfilePictureId = Id<UserProfilePicture>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection, PartialEq, Eq)]
#[mongo(collection_name = "user_profile_pictures")]
#[mongo(index(fields(user_id = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct UserProfilePicture {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: UserProfilePictureId,
	pub user_id: UserId,
	pub image_set: ImageSet,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	std::iter::once(MongoGenericCollection::new::<UserProfilePicture>())
}
