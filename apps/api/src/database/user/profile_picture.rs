use ulid::Ulid;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserProfilePicture {
	pub id: Ulid,
	pub user_id: Ulid,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserProfilePictureFile {
	pub user_profile_picture_id: Ulid,
	pub file_id: Ulid,
	// TODO: Add more fields to describe the file
}
