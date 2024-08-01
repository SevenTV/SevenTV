use crate::database::{
	event::{
		EventBadgeData, EventEmoteData, EventEmoteModerationRequestData, EventEmoteSetData,
		EventEntitlementEdgeData, EventPaintData, EventRoleData, EventTicketData, EventTicketMessageData,
		EventUserBanData, EventUserData, EventUserEditorData, EventUserSessionData,
	},
	badge::Badge,
	emote::Emote,
	emote_moderation_request::EmoteModerationRequest,
	emote_set::EmoteSet,
	entitlement::EntitlementEdge,
	paint::Paint,
	role::Role,
	ticket::{Ticket, TicketMessage},
	user::{ban::UserBan, editor::UserEditor, session::UserSession, User},
};

pub enum EventPayload {
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
