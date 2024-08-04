use crate::{
	database::{
		badge::Badge,
		emote::Emote,
		emote_moderation_request::EmoteModerationRequest,
		emote_set::EmoteSet,
		entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind},
		event::{
			Event, EventBadgeData, EventData, EventEmoteData, EventEmoteModerationRequestData, EventEmoteSetData,
			EventEntitlementEdgeData, EventId, EventPaintData, EventRoleData, EventTicketData, EventTicketMessageData,
			EventUserBanData, EventUserData, EventUserEditorData, EventUserProfilePictureData, EventUserSessionData,
		},
		paint::Paint,
		role::Role,
		ticket::{Ticket, TicketMessage},
		user::{ban::UserBan, editor::UserEditor, profile_picture::UserProfilePicture, session::UserSession, User, UserId},
		Id,
	},
	event_api,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub struct EventPayload {
	pub actor_id: Option<UserId>,
	pub data: EventPayloadData,
	pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventPayloadData {
	Emote {
		after: Emote,
		data: EventEmoteData,
	},
	EmoteSet {
		after: EmoteSet,
		data: EventEmoteSetData,
	},
	User {
		after: User,
		data: EventUserData,
	},
	UserProfilePicture {
		after: UserProfilePicture,
		data: EventUserProfilePictureData,
	},
	UserEditor {
		after: UserEditor,
		data: EventUserEditorData,
	},
	UserBan {
		after: UserBan,
		data: EventUserBanData,
	},
	UserSession {
		after: UserSession,
		data: EventUserSessionData,
	},
	Ticket {
		after: Ticket,
		data: EventTicketData,
	},
	TicketMessage {
		after: TicketMessage,
		data: EventTicketMessageData,
	},
	EmoteModerationRequest {
		after: EmoteModerationRequest,
		data: EventEmoteModerationRequestData,
	},
	Paint {
		after: Paint,
		data: EventPaintData,
	},
	Badge {
		after: Badge,
		data: EventBadgeData,
	},
	Role {
		after: Role,
		data: EventRoleData,
	},
	EntitlementEdge {
		after: EntitlementEdge,
		data: EventEntitlementEdgeData,
	},
}

impl EventPayloadData {
	pub fn subject(&self) -> &'static str {
		match self {
			EventPayloadData::Emote { .. } => "emote",
			EventPayloadData::EmoteSet { .. } => "emote_set",
			EventPayloadData::User { .. } => "user",
			EventPayloadData::UserProfilePicture { .. } => "user_profile_picture",
			EventPayloadData::UserEditor { .. } => "user_editor",
			EventPayloadData::UserBan { .. } => "user_ban",
			EventPayloadData::UserSession { .. } => "user_session",
			EventPayloadData::Ticket { .. } => "ticket",
			EventPayloadData::TicketMessage { .. } => "ticket_message",
			EventPayloadData::EmoteModerationRequest { .. } => "emote_moderation_request",
			EventPayloadData::Paint { .. } => "paint",
			EventPayloadData::Badge { .. } => "badge",
			EventPayloadData::Role { .. } => "role",
			EventPayloadData::EntitlementEdge { .. } => "entitlement_edge",
		}
	}

	pub fn id(&self) -> Option<Id> {
		let id = match self {
			EventPayloadData::Emote { after, .. } => after.id.cast(),
			EventPayloadData::EmoteSet { after, .. } => after.id.cast(),
			EventPayloadData::User { after, .. } => after.id.cast(),
			EventPayloadData::UserProfilePicture { after, .. } => after.id.cast(),
			EventPayloadData::UserEditor { after, .. } => after.id.user_id.cast(),
			EventPayloadData::UserBan { after, .. } => after.id.cast(),
			EventPayloadData::UserSession { after, .. } => after.id.cast(),
			EventPayloadData::Ticket { after, .. } => after.id.cast(),
			EventPayloadData::TicketMessage { after, .. } => after.id.cast(),
			EventPayloadData::EmoteModerationRequest { after, .. } => after.id.cast(),
			EventPayloadData::Paint { after, .. } => after.id.cast(),
			EventPayloadData::Badge { after, .. } => after.id.cast(),
			EventPayloadData::Role { after, .. } => after.id.cast(),
			// only for role assignments
			EventPayloadData::EntitlementEdge {
				after:
					EntitlementEdge {
						id:
							EntitlementEdgeId {
								from: EntitlementEdgeKind::User { user_id },
								to: EntitlementEdgeKind::Role { .. },
								..
							},
					},
				..
			} => user_id.cast(),
			_ => return None,
		};

		Some(id)
	}

	pub fn event_api_kind(&self) -> event_api::types::ObjectKind {
		match self {
			EventPayloadData::Emote { .. } => event_api::types::ObjectKind::Emote,
			EventPayloadData::EmoteSet { .. } => event_api::types::ObjectKind::EmoteSet,
			EventPayloadData::User { .. } => event_api::types::ObjectKind::User,
			EventPayloadData::UserProfilePicture { .. } => event_api::types::ObjectKind::User,
			EventPayloadData::UserEditor { .. } => event_api::types::ObjectKind::User,
			EventPayloadData::UserBan { .. } => event_api::types::ObjectKind::User,
			EventPayloadData::UserSession { .. } => event_api::types::ObjectKind::User,
			EventPayloadData::Ticket { .. } => event_api::types::ObjectKind::Report,
			EventPayloadData::TicketMessage { .. } => event_api::types::ObjectKind::Report,
			EventPayloadData::EmoteModerationRequest { .. } => event_api::types::ObjectKind::Message,
			EventPayloadData::Paint { .. } => event_api::types::ObjectKind::Cosmetic,
			EventPayloadData::Badge { .. } => event_api::types::ObjectKind::Cosmetic,
			EventPayloadData::Role { .. } => event_api::types::ObjectKind::Role,
			EventPayloadData::EntitlementEdge { .. } => event_api::types::ObjectKind::User,
		}
	}

	pub fn event_api_event_type(&self) -> Option<event_api::types::EventType> {
		let ty = match self {
			EventPayloadData::Emote { .. } => event_api::types::EventType::UpdateEmote,
			EventPayloadData::EmoteSet {
				data: EventEmoteSetData::Delete,
				..
			} => event_api::types::EventType::DeleteEmoteSet,
			EventPayloadData::EmoteSet { .. } => event_api::types::EventType::UpdateEmoteSet,
			EventPayloadData::User { .. } => event_api::types::EventType::UpdateUser,
			EventPayloadData::UserProfilePicture { .. } => event_api::types::EventType::UpdateUser,
			EventPayloadData::UserEditor { .. } => event_api::types::EventType::UpdateUser,
			EventPayloadData::UserBan { .. } => event_api::types::EventType::UpdateUser,
			EventPayloadData::UserSession { .. } => event_api::types::EventType::UpdateUser,
			EventPayloadData::EntitlementEdge { .. } => event_api::types::EventType::UpdateUser,
			_ => return None,
		};

		Some(ty)
	}
}

impl From<EventPayload> for Event {
	fn from(payload: EventPayload) -> Self {
		let data = match payload.data {
			EventPayloadData::Emote { after, data } => EventData::Emote {
				target_id: after.id,
				data,
			},
			EventPayloadData::EmoteSet { after, data } => EventData::EmoteSet {
				target_id: after.id,
				data,
			},
			EventPayloadData::User { after, data } => EventData::User {
				target_id: after.id,
				data,
			},
			EventPayloadData::UserProfilePicture { after, data } => EventData::UserProfilePicture {
				target_id: after.id,
				data,
			},
			EventPayloadData::UserEditor { after, data } => EventData::UserEditor {
				target_id: after.id,
				data,
			},
			EventPayloadData::UserBan { after, data } => EventData::UserBan {
				target_id: after.id,
				data,
			},
			EventPayloadData::UserSession { after, data } => EventData::UserSession {
				target_id: after.id,
				data,
			},
			EventPayloadData::Ticket { after, data } => EventData::Ticket {
				target_id: after.id,
				data,
			},
			EventPayloadData::TicketMessage { after, data } => EventData::TicketMessage {
				target_id: after.id,
				data,
			},
			EventPayloadData::EmoteModerationRequest { after, data } => EventData::EmoteModerationRequest {
				target_id: after.id,
				data,
			},
			EventPayloadData::Paint { after, data } => EventData::Paint {
				target_id: after.id,
				data,
			},
			EventPayloadData::Badge { after, data } => EventData::Badge {
				target_id: after.id,
				data,
			},
			EventPayloadData::Role { after, data } => EventData::Role {
				target_id: after.id,
				data,
			},
			EventPayloadData::EntitlementEdge { after, data } => EventData::EntitlementEdge {
				target_id: after.id,
				data,
			},
		};

		Self {
			id: EventId::with_timestamp(payload.timestamp),
			actor_id: payload.actor_id,
			data,
			updated_at: payload.timestamp,
			search_updated_at: None,
		}
	}
}
