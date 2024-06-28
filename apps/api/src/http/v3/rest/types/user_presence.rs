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

#[derive(Debug, Clone, Copy, Default, utoipa::ToSchema, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user-presence.model.go#L19
pub enum PresenceKind {
	#[default]
	UserPresenceKindUnknown = 0,
	UserPresenceKindChannel = 1,
	UserPresenceKindWebPage = 2,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct UserPresenceWriteResponse {
	pub ok: bool,
	pub presence: PresenceModel,
}
