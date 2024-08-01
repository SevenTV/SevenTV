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
		before: Emote,
		data: AuditLogEmoteData,
	},
	EmoteSet {
		before: EmoteSet,
		data: AuditLogEmoteSetData,
	},
	User {
		before: User,
		data: AuditLogUserData,
	},
	UserEditor {
		before: UserEditor,
		data: AuditLogUserEditorData,
	},
	UserBan {
		before: UserBan,
		data: AuditLogUserBanData,
	},
	UserSession {
		before: UserSession,
		data: AuditLogUserSessionData,
	},
	Ticket {
		before: Ticket,
		data: AuditLogTicketData,
	},
	TicketMessage {
		before: TicketMessage,
		data: AuditLogTicketMessageData,
	},
	EmoteModerationRequest {
		before: EmoteModerationRequest,
		data: AuditLogEmoteModerationRequestData,
	},
	Paint {
		before: Paint,
		data: AuditLogPaintData,
	},
	Badge {
		before: Badge,
		data: AuditLogBadgeData,
	},
	Role {
		before: Role,
		data: AuditLogRoleData,
	},
	EntitlementEdge {
		before: EntitlementEdge,
		data: AuditLogEntitlementEdgeData,
	},
}
