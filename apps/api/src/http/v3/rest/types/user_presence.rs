use shared::database::user::UserId;
use shared::database::Id;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user-presence.model.go#L9
pub struct PresenceModel {
	pub id: Id<()>,
	pub user_id: UserId,
	pub timestamp: i64,
	pub ttl: i64,
	pub kind: PresenceKind,
}

#[derive(
	Debug, Clone, Copy, Default, Eq, PartialEq, utoipa::ToSchema, serde_repr::Serialize_repr, serde_repr::Deserialize_repr,
)]
#[repr(u8)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user-presence.model.go#L19
pub enum PresenceKind {
	#[default]
	Unknown = 0,
	Channel = 1,
	WebPage = 2,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct UserPresenceWriteResponse {
	pub ok: bool,
	pub presence: PresenceModel,
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
#[serde(default)]
pub struct UserPresenceWriteRequest {
	pub kind: PresenceKind,
	pub session_id: Option<String>,
	pub passive: bool,
	pub data: UserPresenceWriteData,
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
#[serde(default)]
pub struct UserPresenceWriteData {
	pub platform: UserPresencePlatform,
	pub id: String,
}

#[derive(Debug, Default, Clone, Copy, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserPresencePlatform {
	#[default]
	Twitch,
	Kick,
	Youtube,
}
