use crate::database::{
	audit_log::{
		AuditLogBadgeData, AuditLogEmoteData, AuditLogEmoteModerationRequestData, AuditLogEmoteSetData,
		AuditLogEntitlementEdgeData, AuditLogPaintData, AuditLogRoleData, AuditLogTicketData, AuditLogTicketMessageData,
		AuditLogUserBanData, AuditLogUserData, AuditLogUserEditorData, AuditLogUserSessionData,
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

pub enum Event {
	Emote {
		after: Emote,
		data: AuditLogEmoteData,
	},
	EmoteSet {
		after: EmoteSet,
		data: AuditLogEmoteSetData,
	},
	User {
		after: User,
		data: AuditLogUserData,
	},
	UserEditor {
		after: UserEditor,
		data: AuditLogUserEditorData,
	},
	UserBan {
		after: UserBan,
		data: AuditLogUserBanData,
	},
	UserSession {
		after: UserSession,
		data: AuditLogUserSessionData,
	},
	Ticket {
		after: Ticket,
		data: AuditLogTicketData,
	},
	TicketMessage {
		after: TicketMessage,
		data: AuditLogTicketMessageData,
	},
	EmoteModerationRequest {
		after: EmoteModerationRequest,
		data: AuditLogEmoteModerationRequestData,
	},
	Paint {
		after: Paint,
		data: AuditLogPaintData,
	},
	Badge {
		after: Badge,
		data: AuditLogBadgeData,
	},
	Role {
		after: Role,
		data: AuditLogRoleData,
	},
	EntitlementEdge {
		after: EntitlementEdge,
		data: AuditLogEntitlementEdgeData,
	},
}
