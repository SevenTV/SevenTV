use std::collections::HashSet;

use mongodb::bson::oid::ObjectId;
use shared::database::user::connection::Platform;

use crate::types;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("found duplicate user connection")]
	DuplicateUserConnection { platform: Platform, platform_id: String },
	#[error("user connection without platform id")]
	MissingPlatformId { user_id: ObjectId, platform: Platform },
	#[error("mongo failed: {0}")]
	Mongo(#[from] mongodb::error::Error),
	#[error("failed to serialize json data")]
	SerializeJson(#[from] serde_json::Error),
	#[error("failed to query clickhouse")]
	Clickhouse(#[from] clickhouse::error::Error),

	#[error("emote set emote found with no name")]
	EmoteSetEmoteNoName { emote_set_id: ObjectId, emote_id: ObjectId },

	#[error("unsupported audit log kind")]
	UnsupportedAuditLogKind(types::AuditLogKind),
	#[error("failed to convert timestamp")]
	Timestamp(#[from] time::error::ComponentRange),

	#[error("invalid stripe id: {0}")]
	InvalidStripeId(stripe::ParseIdError),
	#[error("{0}")]
	Stripe(#[from] stripe::StripeError),
	#[error("invalid subscription cycle")]
	InvalidSubscriptionCycle,

	// #[error("duplicate emote moderation request")]
	// DuplicateEmoteModRequest {
	// 	emote_id: EmoteId,
	// 	kind: EmoteModerationRequestKind,
	// },
	#[error("reqwest error")]
	Reqwest(#[from] reqwest::Error),
	#[error("failed to read image")]
	Io(#[from] std::io::Error),
	#[error("invalid cdn file")]
	InvalidCdnFile(anyhow::Error),
	#[error("failed to download image")]
	ImageDownload {
		cosmetic_id: ObjectId,
		status: reqwest::StatusCode,
	},
	#[error("grpc error")]
	Grpc(#[from] tonic::Status),
	#[error("image processor error")]
	ImageProcessor(scuffle_image_processor_proto::Error),
	#[error("failed to set once cell value")]
	OnceCellSet(#[from] tokio::sync::SetError<HashSet<String>>),

	#[error("not implemented")]
	NotImplemented(&'static str),

	#[error("redeem code: {0}")]
	RedeemCode(String),

	#[error("failed to rename cdn file: {0:#}")]
	CdnRename(anyhow::Error),

	#[error("failed to run emote stats: {0:#}")]
	EmoteStats(anyhow::Error),
}

impl Error {
	pub fn name(&self) -> &'static str {
		match self {
			Self::DuplicateUserConnection { .. } => "DuplicateUserConnection",
			Self::MissingPlatformId { .. } => "MissingPlatformId",
			Self::Mongo(_) => "Deserialize",
			Self::SerializeJson(_) => "SerializeJson",
			Self::Clickhouse(_) => "Clickhouse",
			Self::EmoteSetEmoteNoName { .. } => "EmoteSetEmoteNoName",
			Self::UnsupportedAuditLogKind(_) => "UnsupportedAuditLogKind",
			Self::Timestamp(_) => "Timestamp",
			Self::Stripe(_) => "Stripe",
			Self::InvalidSubscriptionCycle => "InvalidSubscriptionCycle",
			Self::InvalidStripeId(_) => "InvalidStripeId",
			Self::Reqwest(_) => "Reqwest",
			Self::Io(_) => "Io",
			Self::InvalidCdnFile(_) => "InvalidCdnFile",
			Self::ImageDownload { .. } => "ImageDownload",
			Self::Grpc(_) => "Grpc",
			Self::ImageProcessor(_) => "ImageProcessor",
			Self::OnceCellSet(_) => "OnceCellSet",
			Self::NotImplemented(_) => "NotImplemented",
			Self::RedeemCode(_) => "RedeemCode",
			Self::CdnRename(_) => "CdnRename",
			Self::EmoteStats(_) => "EmoteStats",
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
			Self::ImageProcessor(e) => e.message.clone(),
			Self::InvalidCdnFile(e) => format!("{}", e),
			Self::ImageDownload { cosmetic_id, status } => format!("cosmetic id: {}, status: {}", cosmetic_id, status),
			// Self::DuplicateEmoteModRequest { emote_id, kind } => format!("emote id: {}, kind: {:?}", emote_id, kind),
			Self::NotImplemented(msg) => msg.to_string(),
			e => format!("{:?}", e),
		}
	}
}
