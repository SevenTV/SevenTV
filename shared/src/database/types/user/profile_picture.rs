use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct UserProfilePicture {
	pub id: ulid::Ulid,
	pub user_id: ulid::Ulid,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct UserProfilePictureFile {
	pub user_profile_picture_id: ulid::Ulid,
	pub file_id: ulid::Ulid,
	// TODO: Add more fields to describe the file
}

impl Table for UserProfilePicture {
	const TABLE_NAME: &'static str = "user_profile_pictures";
}
