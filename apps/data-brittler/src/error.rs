use mongodb::bson::oid::ObjectId;
use shared::database::Platform;

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
	Db(#[from] mongodb::error::Error),
	#[error("failed to insert all documents")]
	InsertMany,
	#[error("failed to query clickhouse")]
	Clickhouse(#[from] clickhouse::error::Error),

	#[error("emote set emote found with no name")]
	EmoteSetEmoteNoName { emote_set_id: ObjectId, emote_id: ObjectId },

	#[error("unsupported audit log kind")]
	UnsupportedAuditLogKind(types::AuditLogKind),
	#[error("failed to convert timestamp")]
	Timestamp(#[from] time::error::ComponentRange),

	#[error("invalid stripe id: {0}")]
	InvalidStripeId(String),
	#[error("{0}")]
	Stripe(#[from] stripe::StripeError),

	#[error("not implemented")]
	NotImplemented(&'static str),
}

impl Error {
	pub fn name(&self) -> &'static str {
		match self {
			Self::DuplicateUserConnection { .. } => "DuplicateUserConnection",
			Self::MissingPlatformId { .. } => "MissingPlatformId",
			Self::Deserialize(_) => "Deserialize",
			Self::SerializeJson(_) => "SerializeJson",
			Self::Db(_) => "Db",
			Self::InsertMany => "InsertMany",
			Self::Clickhouse(_) => "Clickhouse",
			Self::EmoteSetEmoteNoName { .. } => "EmoteSetEmoteNoName",
			Self::UnsupportedAuditLogKind(_) => "UnsupportedAuditLogKind",
			Self::Timestamp(_) => "Timestamp",
			Self::Stripe(_) => "Stripe",
			Self::InvalidStripeId(_) => "InvalidStripeId",
			Self::NotImplemented(_) => "NotImplemented",
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
			Self::EmoteSetEmoteNoName { emote_set_id, emote_id } => {
				format!("emote set id: {}, emote id: {:?}", emote_set_id, emote_id)
			}
			Self::UnsupportedAuditLogKind(kind) => format!("kind: {:?}", kind),
			Self::InvalidStripeId(id) => format!("id: {}", id),
			Self::NotImplemented(msg) => msg.to_string(),
			e => format!("{:?}", e),
		}
	}
}
