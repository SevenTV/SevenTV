use shared::database::Platform;
use shared::object_id::ObjectId;

use crate::types;

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
	#[error("failed to query clickhouse")]
	Clickhouse(#[from] clickhouse::error::Error),

	#[error("{0}")]
	ImageFile(#[from] crate::types::ImageFileError),

	#[error("failed to fetch paint image url")]
	PaintImageUrlRequest(reqwest::Error),

	#[error("unsupported audit log kind")]
	UnsupportedAuditLogKind(types::AuditLogKind),
	#[error("failed to convert timestamp")]
	Timestamp(#[from] time::error::ComponentRange),
}

impl Error {
	pub fn name(&self) -> &'static str {
		match self {
			Self::DuplicateUserConnection { .. } => "DuplicateUserConnection",
			Self::MissingPlatformId { .. } => "MissingPlatformId",
			Self::Deserialize(_) => "Deserialize",
			Self::SerializeJson(_) => "SerializeJson",
			Self::Db(_) => "Db",
			Self::Clickhouse(_) => "Clickhouse",
			Self::ImageFile(_) => "ImageFile",
			Self::PaintImageUrlRequest(_) => "PaintImageUrlRequest",
			Self::UnsupportedAuditLogKind(_) => "UnsupportedAuditLogKind",
			Self::Timestamp(_) => "Timestamp",
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
			Self::UnsupportedAuditLogKind(kind) => format!("kind: {:?}", kind),
			e => format!("{:?}", e),
		}
	}
}
