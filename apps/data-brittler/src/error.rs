use shared::database::Platform;
use shared::object_id::ObjectId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("found duplicate user connection")]
	DuplicateUserConnection { platform: Platform, platform_id: String },
	#[error("user connection without platform id")]
	MissingPlatformId { user_id: ObjectId, platform: Platform },
	#[error("failed to deserialize")]
	Deserialize(mongodb::error::Error),
	#[error("failed to serialize json data")]
	SerializeJson(#[from] serde_json::Error),
	#[error("failed to query database")]
	Db(#[from] tokio_postgres::Error),

	#[error("{0}")]
	ImageFile(#[from] crate::types::ImageFileError),

	#[error("failed to fetch paint image url")]
	PaintImageUrlRequest(reqwest::Error),
}

impl Error {
	pub fn name(&self) -> &'static str {
		match self {
			Self::DuplicateUserConnection { .. } => "DuplicateUserConnection",
			Self::MissingPlatformId { .. } => "MissingPlatformId",
			Self::Deserialize(_) => "Deserialize",
			Self::SerializeJson(_) => "SerializeJson",
			Self::Db(_) => "Db",
			Self::ImageFile(_) => "ImageFile",
			Self::PaintImageUrlRequest(_) => "PaintImageUrlRequest",
		}
	}

	pub fn data(&self) -> String {
		match self {
			Self::DuplicateUserConnection { platform, platform_id } => {
				format!("platform: {:?}, platform id: {}", platform, platform_id)
			}
			Self::MissingPlatformId { user_id, platform } => {
				format!("user id: {}, platform: {:?}", user_id, platform)
			}
			e => format!("{:?}", e),
		}
	}
}
