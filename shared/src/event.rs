use std::collections::HashMap;

use anyhow::Context;

use crate::database::badge::Badge;
use crate::database::emote::{Emote, EmoteFlags};
use crate::database::emote_moderation_request::EmoteModerationRequest;
use crate::database::emote_set::{EmoteSet, EmoteSetEmote};
use crate::database::entitlement::EntitlementEdgeKind;
use crate::database::paint::Paint;
use crate::database::role::Role;
use crate::database::stored_event::{
	StoredEvent, StoredEventBadgeData, StoredEventData, StoredEventEmoteData, StoredEventEmoteModerationRequestData,
	StoredEventEmoteSetData, StoredEventId, StoredEventPaintData, StoredEventRoleData, StoredEventTicketData,
	StoredEventTicketMessageData, StoredEventUserBanData, StoredEventUserData, StoredEventUserEditorData,
	StoredEventUserProfilePictureData, StoredEventUserSessionData,
};
use crate::database::ticket::{Ticket, TicketMessage, TicketPriority};
use crate::database::user::ban::UserBan;
use crate::database::user::connection::UserConnection;
use crate::database::user::editor::{UserEditor, UserEditorPermissions};
use crate::database::user::profile_picture::UserProfilePicture;
use crate::database::user::session::{UserSession, UserSessionId};
use crate::database::user::{FullUser, User, UserId};
use crate::database::Id;
use crate::event_api::types::{ChangeField, ChangeFieldType};
use crate::event_api::{self};
use crate::old_types::{
	ActiveEmoteModel, EmoteFlagsModel, EmotePartialModel, EmoteSetModel, UserConnectionModel, UserConnectionPartialModel,
	UserPartialModel,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct InternalEventPayload {
	pub events: Vec<InternalEvent>,
	pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl InternalEventPayload {
	pub fn new(events: impl IntoIterator<Item = InternalEvent>) -> Self {
		Self {
			events: events.into_iter().collect(),
			timestamp: chrono::Utc::now(),
		}
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct InternalEvent {
	pub actor: Option<FullUser>,
	pub session_id: Option<UserSessionId>,
	pub data: InternalEventData,
	pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl InternalEvent {
	pub fn kind(&self) -> &'static str {
		match &self.data {
			InternalEventData::Emote { data, .. } => match data {
				StoredEventEmoteData::Upload => "emote.upload",
				StoredEventEmoteData::ChangeName { .. } => "emote.change_name",
				StoredEventEmoteData::ChangeFlags { .. } => "emote.change_flags",
				StoredEventEmoteData::ChangeOwner { .. } => "emote.change_owner",
				StoredEventEmoteData::ChangeTags { .. } => "emote.change_tags",
				StoredEventEmoteData::Process { .. } => "emote.process",
				StoredEventEmoteData::Delete => "emote.delete",
				StoredEventEmoteData::Merge { .. } => "emote.merge",
			},
			InternalEventData::EmoteSet { data, .. } => match data {
				InternalEventEmoteSetData::Create => "emote_set.create",
				InternalEventEmoteSetData::ChangeName { .. } => "emote_set.change_name",
				InternalEventEmoteSetData::ChangeTags { .. } => "emote_set.change_tags",
				InternalEventEmoteSetData::ChangeCapacity { .. } => "emote_set.change_capacity",
				InternalEventEmoteSetData::AddEmote { .. } => "emote_set.add_emote",
				InternalEventEmoteSetData::RemoveEmote { .. } => "emote_set.remove_emote",
				InternalEventEmoteSetData::RenameEmote { .. } => "emote_set.rename_emote",
				InternalEventEmoteSetData::Delete => "emote_set.delete",
			},
			InternalEventData::User { data, .. } => match data {
				InternalEventUserData::Create => "user.create",
				InternalEventUserData::ChangeActivePaint { .. } => "user.change_active_paint",
				InternalEventUserData::ChangeActiveBadge { .. } => "user.change_active_badge",
				InternalEventUserData::ChangeActiveEmoteSet { .. } => "user.change_active_emote_set",
				InternalEventUserData::AddConnection { .. } => "user.add_connection",
				InternalEventUserData::RemoveConnection { .. } => "user.remove_connection",
				InternalEventUserData::Merge { .. } => "user.merge",
				InternalEventUserData::Delete => "user.delete",
				InternalEventUserData::AddEntitlement { .. } => "user.add_entitlement",
				InternalEventUserData::RemoveEntitlement { .. } => "user.remove_entitlement",
			},
			InternalEventData::UserProfilePicture { data, .. } => match data {
				StoredEventUserProfilePictureData::Create => "user_profile_picture.create",
				StoredEventUserProfilePictureData::Process { .. } => "user_profile_picture.process",
			},
			InternalEventData::UserEditor { data, .. } => match data {
				InternalEventUserEditorData::AddEditor { .. } => "user_editor.add_editor",
				InternalEventUserEditorData::RemoveEditor { .. } => "user_editor.remove_editor",
				InternalEventUserEditorData::EditPermissions { .. } => "user_editor.edit_permissions",
			},
			InternalEventData::UserBan { data, .. } => match data {
				StoredEventUserBanData::Ban => "user_ban.ban",
				StoredEventUserBanData::ChangeExpiresAt { .. } => "user_ban.change_expires_at",
				StoredEventUserBanData::ChangeReason { .. } => "user_ban.change_reason",
				StoredEventUserBanData::ChangeUserBanPermissions { .. } => "user_ban.change_user_ban_permissions",
				StoredEventUserBanData::Unban => "user_ban.unban",
			},
			InternalEventData::UserSession { data, .. } => match data {
				StoredEventUserSessionData::Create { .. } => "user_session.create",
				StoredEventUserSessionData::Delete => "user_session.delete",
			},
			InternalEventData::Ticket { data, .. } => match data {
				InternalEventTicketData::Create => "ticket.create",
				InternalEventTicketData::AddMember { .. } => "ticket.add_member",
				InternalEventTicketData::RemoveMember { .. } => "ticket.remove_member",
				InternalEventTicketData::ChangeOpen { .. } => "ticket.change_open",
				InternalEventTicketData::ChangePriority { .. } => "ticket.change_priority",
			},
			InternalEventData::TicketMessage { data, .. } => match data {
				StoredEventTicketMessageData::Create => "ticket_message.create",
			},
			InternalEventData::EmoteModerationRequest { data, .. } => match data {
				StoredEventEmoteModerationRequestData::Create => "emote_moderation_request.create",
				StoredEventEmoteModerationRequestData::ChangeStatus { .. } => "emote_moderation_request.change_status",
			},
			InternalEventData::Paint { data, .. } => match data {
				StoredEventPaintData::Create => "paint.create",
				StoredEventPaintData::ChangeData { .. } => "paint.change_data",
				StoredEventPaintData::ChangeName { .. } => "paint.change_name",
				StoredEventPaintData::Process { .. } => "paint.process",
			},
			InternalEventData::Badge { data, .. } => match data {
				StoredEventBadgeData::Create => "badge.create",
				StoredEventBadgeData::Process { .. } => "badge.process",
			},
			InternalEventData::Role { data, .. } => match data {
				StoredEventRoleData::Create => "role.create",
				StoredEventRoleData::AddEntitlement { .. } => "role.add_entitlement",
				StoredEventRoleData::RemoveEntitlement { .. } => "role.remove_entitlement",
				StoredEventRoleData::ChangeName { .. } => "role.change_name",
				StoredEventRoleData::ChangePermissions { .. } => "role.change_permissions",
				StoredEventRoleData::Delete => "role.delete",
				StoredEventRoleData::ChangeColor { .. } => "role.change_color",
				StoredEventRoleData::ChangeRank { .. } => "role.change_rank",
			},
			InternalEventData::UserPresence(_) => "user_presence.create",
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventEmoteKind {
	ChangeName,
	ChangeFlags,
	ChangeOwner,
	ChangeTags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventEmoteSetKind {
	Create,
	ChangeName,
	ChangeTags,
	ChangeCapacity,
	AddEmote,
	RemoveEmote,
	RenameEmote,
	Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventUserKind {
	Create,
	ChangeActivePaint,
	ChangeActiveBadge,
	ChangeActiveEmoteSet,
	AddConnection,
	RemoveConnection,
	Merge,
	Delete,
	AddEntitlement,
	RemoveEntitlement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventUserProfilePictureKind {
	Process,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventUserEditorKind {
	AddEditor,
	RemoveEditor,
	EditPermissions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventUserBanKind {
	Ban,
	Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventUserSessionKind {
	Create,
	Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventTicketKind {
	Create,
	AddMember,
	RemoveMember,
	ChangeOpen,
	ChangePriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventTicketMessageKind {
	Create,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventEmoteModerationRequestKind {
	Create,
	Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventPaintKind {
	Create,
	Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventBadgeKind {
	Create,
	Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventRoleKind {
	Create,
	Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalEventUserPresenceKind {
	Update,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum EventUserPresencePlatform {
	Twitch(u32),
	Kick(u32),
	Youtube(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InternalEventData {
	Emote {
		after: Emote,
		data: StoredEventEmoteData,
	},
	EmoteSet {
		after: EmoteSet,
		data: InternalEventEmoteSetData,
	},
	User {
		after: User,
		data: InternalEventUserData,
	},
	UserPresence(Box<InternalEventUserPresenceData>),
	UserProfilePicture {
		after: UserProfilePicture,
		data: StoredEventUserProfilePictureData,
	},
	UserEditor {
		after: UserEditor,
		data: InternalEventUserEditorData,
	},
	UserBan {
		after: UserBan,
		data: StoredEventUserBanData,
	},
	UserSession {
		after: UserSession,
		data: StoredEventUserSessionData,
	},
	Ticket {
		after: Ticket,
		data: InternalEventTicketData,
	},
	TicketMessage {
		after: TicketMessage,
		data: StoredEventTicketMessageData,
	},
	EmoteModerationRequest {
		after: EmoteModerationRequest,
		data: StoredEventEmoteModerationRequestData,
	},
	Paint {
		after: Paint,
		data: StoredEventPaintData,
	},
	Badge {
		after: Badge,
		data: StoredEventBadgeData,
	},
	Role {
		after: Role,
		data: StoredEventRoleData,
	},
}

impl InternalEventData {
	pub fn id(&self) -> Id {
		match self {
			InternalEventData::Emote { after, .. } => after.id.cast(),
			InternalEventData::EmoteSet { after, .. } => after.id.cast(),
			InternalEventData::User { after, .. } => after.id.cast(),
			InternalEventData::UserProfilePicture { after, .. } => after.user_id.cast(),
			InternalEventData::UserEditor { after, .. } => after.id.user_id.cast(),
			InternalEventData::UserBan { after, .. } => after.user_id.cast(),
			InternalEventData::UserSession { after, .. } => after.user_id.cast(),
			InternalEventData::Ticket { after, .. } => after.id.cast(),
			InternalEventData::TicketMessage { after, .. } => after.ticket_id.cast(),
			InternalEventData::EmoteModerationRequest { after, .. } => after.id.cast(),
			InternalEventData::Paint { after, .. } => after.id.cast(),
			InternalEventData::Badge { after, .. } => after.id.cast(),
			InternalEventData::Role { after, .. } => after.id.cast(),
			InternalEventData::UserPresence(data) => data.user.id.cast(),
		}
	}

	pub fn event_api_kind(&self) -> event_api::types::ObjectKind {
		match self {
			InternalEventData::Emote { .. } => event_api::types::ObjectKind::Emote,
			InternalEventData::EmoteSet { .. } => event_api::types::ObjectKind::EmoteSet,
			InternalEventData::User { .. } => event_api::types::ObjectKind::User,
			InternalEventData::UserProfilePicture { .. } => event_api::types::ObjectKind::User,
			InternalEventData::UserEditor { .. } => event_api::types::ObjectKind::User,
			InternalEventData::UserBan { .. } => event_api::types::ObjectKind::User,
			InternalEventData::UserSession { .. } => event_api::types::ObjectKind::User,
			InternalEventData::Ticket { .. } => event_api::types::ObjectKind::Report,
			InternalEventData::TicketMessage { .. } => event_api::types::ObjectKind::Report,
			InternalEventData::EmoteModerationRequest { .. } => event_api::types::ObjectKind::Message,
			InternalEventData::Paint { .. } => event_api::types::ObjectKind::Cosmetic,
			InternalEventData::Badge { .. } => event_api::types::ObjectKind::Cosmetic,
			InternalEventData::Role { .. } => event_api::types::ObjectKind::Role,
			InternalEventData::UserPresence { .. } => event_api::types::ObjectKind::User,
		}
	}

	pub fn event_api_event_type(&self) -> Option<event_api::types::EventType> {
		let ty = match self {
			InternalEventData::Emote { .. } => event_api::types::EventType::UpdateEmote,
			InternalEventData::EmoteSet {
				data: InternalEventEmoteSetData::Delete,
				..
			} => event_api::types::EventType::DeleteEmoteSet,
			InternalEventData::EmoteSet { .. } => event_api::types::EventType::UpdateEmoteSet,
			InternalEventData::User { .. } => event_api::types::EventType::UpdateUser,
			InternalEventData::UserProfilePicture { .. } => event_api::types::EventType::UpdateUser,
			InternalEventData::UserEditor { .. } => event_api::types::EventType::UpdateUser,
			InternalEventData::UserBan { .. } => event_api::types::EventType::UpdateUser,
			InternalEventData::UserSession { .. } => event_api::types::EventType::UpdateUser,
			_ => return None,
		};

		Some(ty)
	}
}

impl TryFrom<InternalEvent> for StoredEvent {
	type Error = ();

	fn try_from(payload: InternalEvent) -> Result<Self, Self::Error> {
		let data = match payload.data {
			InternalEventData::Emote { after, data } => StoredEventData::Emote {
				target_id: after.id,
				data,
			},
			InternalEventData::EmoteSet { after, data } => StoredEventData::EmoteSet {
				target_id: after.id,
				data: data.into(),
			},
			InternalEventData::User { after, data } => StoredEventData::User {
				target_id: after.id,
				data: data.into(),
			},
			InternalEventData::UserProfilePicture { after, data } => StoredEventData::UserProfilePicture {
				target_id: after.id,
				user_id: after.user_id,
				data,
			},
			InternalEventData::UserEditor { after, data } => StoredEventData::UserEditor {
				target_id: after.id,
				data: match data {
					InternalEventUserEditorData::AddEditor { editor } => {
						StoredEventUserEditorData::AddEditor { editor_id: editor.id }
					}
					InternalEventUserEditorData::RemoveEditor { editor } => {
						StoredEventUserEditorData::RemoveEditor { editor_id: editor.id }
					}
					InternalEventUserEditorData::EditPermissions { old, .. } => StoredEventUserEditorData::EditPermissions {
						new: after.permissions,
						old,
					},
				},
			},
			InternalEventData::UserBan { after, data } => StoredEventData::UserBan {
				target_id: after.id,
				user_id: after.user_id,
				data,
			},
			InternalEventData::UserSession { after, data } => StoredEventData::UserSession {
				target_id: after.id,
				user_id: after.user_id,
				data,
			},
			InternalEventData::Ticket { after, data } => StoredEventData::Ticket {
				target_id: after.id,
				data: data.into(),
			},
			InternalEventData::TicketMessage { after, data } => StoredEventData::TicketMessage {
				target_id: after.id,
				ticket_id: after.ticket_id,
				data,
			},
			InternalEventData::EmoteModerationRequest { after, data } => StoredEventData::EmoteModerationRequest {
				target_id: after.id,
				emote_id: after.emote_id,
				data,
			},
			InternalEventData::Paint { after, data } => StoredEventData::Paint {
				target_id: after.id,
				data,
			},
			InternalEventData::Badge { after, data } => StoredEventData::Badge {
				target_id: after.id,
				data,
			},
			InternalEventData::Role { after, data } => StoredEventData::Role {
				target_id: after.id,
				data,
			},
			InternalEventData::UserPresence { .. } => return Err(()),
		};

		Ok(Self {
			id: StoredEventId::with_timestamp(payload.timestamp),
			actor_id: payload.actor.map(|u| u.id),
			data,
			session_id: payload.session_id,
			updated_at: payload.timestamp,
			search_updated_at: None,
		})
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InternalEventUserPresenceData {
	pub user: FullUser,
	pub platform: EventUserPresencePlatform,
	pub active_badge: Option<Badge>,
	pub active_paint: Option<Paint>,
	pub personal_emote_sets: Vec<InternalEventUserPresenceDataEmoteSet>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InternalEventUserPresenceDataEmoteSet {
	pub emote_set: EmoteSet,
	pub emotes: Vec<Emote>,
	#[serde(default)]
	pub emote_owners: HashMap<UserId, FullUser>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum InternalEventEmoteSetData {
	Create,
	ChangeName {
		old: String,
		new: String,
	},
	ChangeTags {
		added: Vec<String>,
		removed: Vec<String>,
	},
	ChangeCapacity {
		old: Option<i32>,
		new: Option<i32>,
	},
	AddEmote {
		emote: Box<Emote>,
		emote_owner: Option<Box<FullUser>>,
		emote_set_emote: EmoteSetEmote,
	},
	RemoveEmote {
		emote: Option<Box<Emote>>,
		emote_owner: Option<Box<FullUser>>,
		emote_set_emote: EmoteSetEmote,
		index: usize,
	},
	RenameEmote {
		emote: Box<Emote>,
		emote_set_emote: EmoteSetEmote,
		old_alias: String,
	},
	Delete,
}

impl From<InternalEventEmoteSetData> for StoredEventEmoteSetData {
	fn from(value: InternalEventEmoteSetData) -> Self {
		match value {
			InternalEventEmoteSetData::Create => StoredEventEmoteSetData::Create,
			InternalEventEmoteSetData::ChangeName { old, new } => StoredEventEmoteSetData::ChangeName { old, new },
			InternalEventEmoteSetData::ChangeCapacity { old, new } => StoredEventEmoteSetData::ChangeCapacity { old, new },
			InternalEventEmoteSetData::ChangeTags { added, removed } => {
				StoredEventEmoteSetData::ChangeTags { added, removed }
			}
			InternalEventEmoteSetData::AddEmote {
				emote, emote_set_emote, ..
			} => StoredEventEmoteSetData::AddEmote {
				emote_id: emote.id,
				alias: emote_set_emote.alias,
			},
			InternalEventEmoteSetData::RemoveEmote { emote_set_emote, .. } => StoredEventEmoteSetData::RemoveEmote {
				emote_id: emote_set_emote.id,
			},
			InternalEventEmoteSetData::RenameEmote {
				emote,
				emote_set_emote,
				old_alias,
			} => StoredEventEmoteSetData::RenameEmote {
				emote_id: emote.id,
				old_alias,
				new_alias: emote_set_emote.alias,
			},
			InternalEventEmoteSetData::Delete => StoredEventEmoteSetData::Delete,
		}
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum InternalEventUserData {
	Create,
	ChangeActivePaint {
		old: Option<Box<Paint>>,
		new: Option<Box<Paint>>,
	},
	ChangeActiveBadge {
		old: Option<Box<Badge>>,
		new: Option<Box<Badge>>,
	},
	ChangeActiveEmoteSet {
		old: Option<Box<EmoteSet>>,
		new: Option<Box<EmoteSet>>,
	},
	AddConnection {
		connection: UserConnection,
	},
	RemoveConnection {
		connection: UserConnection,
	},
	Merge {
		source_id: UserId,
		connections: Vec<UserConnection>,
	},
	Delete,
	AddEntitlement {
		target: EntitlementEdgeKind,
	},
	RemoveEntitlement {
		target: EntitlementEdgeKind,
	},
}

impl From<InternalEventUserData> for StoredEventUserData {
	fn from(value: InternalEventUserData) -> Self {
		match value {
			InternalEventUserData::Create => StoredEventUserData::Create,
			InternalEventUserData::ChangeActivePaint { old, new } => StoredEventUserData::ChangeActivePaint {
				old: old.map(|p| p.id),
				new: new.map(|p| p.id),
			},
			InternalEventUserData::ChangeActiveBadge { old, new } => StoredEventUserData::ChangeActiveBadge {
				old: old.map(|b| b.id),
				new: new.map(|b| b.id),
			},
			InternalEventUserData::ChangeActiveEmoteSet { old, new } => StoredEventUserData::ChangeActiveEmoteSet {
				old: old.map(|e| e.id),
				new: new.map(|e| e.id),
			},
			InternalEventUserData::AddConnection { connection } => StoredEventUserData::AddConnection {
				platform: connection.platform,
			},
			InternalEventUserData::RemoveConnection { connection } => StoredEventUserData::RemoveConnection {
				platform: connection.platform,
			},
			InternalEventUserData::Merge { connections, source_id } => StoredEventUserData::Merge { connections, source_id },
			InternalEventUserData::Delete => StoredEventUserData::Delete,
			InternalEventUserData::AddEntitlement { target } => StoredEventUserData::AddEntitlement { target },
			InternalEventUserData::RemoveEntitlement { target } => StoredEventUserData::RemoveEntitlement { target },
		}
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum InternalEventUserEditorData {
	AddEditor { editor: Box<User> },
	RemoveEditor { editor: Box<User> },
	EditPermissions { editor: Box<User>, old: UserEditorPermissions },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum InternalEventTicketData {
	Create,
	AddMember { member: Box<User> },
	RemoveMember { member: Box<User> },
	ChangeOpen { old: bool, new: bool },
	ChangePriority { old: TicketPriority, new: TicketPriority },
}

impl From<InternalEventTicketData> for StoredEventTicketData {
	fn from(value: InternalEventTicketData) -> Self {
		match value {
			InternalEventTicketData::Create => StoredEventTicketData::Create,
			InternalEventTicketData::AddMember { member } => StoredEventTicketData::AddMember { member: member.id },
			InternalEventTicketData::RemoveMember { member } => StoredEventTicketData::RemoveMember { member: member.id },
			InternalEventTicketData::ChangeOpen { old, new } => StoredEventTicketData::ChangeOpen { old, new },
			InternalEventTicketData::ChangePriority { old, new } => StoredEventTicketData::ChangePriority { old, new },
		}
	}
}

impl InternalEventPayload {
	pub fn into_old_messages(
		self,
		cdn_base_url: &url::Url,
		seq: u64,
	) -> anyhow::Result<(
		Vec<event_api::Message<event_api::payload::Dispatch>>,
		Vec<InternalEventUserPresenceData>,
	)> {
		let mut presence_data = Vec::new();

		let mut events =
			HashMap::<(event_api::types::EventType, Id, event_api::types::ObjectKind), Vec<InternalEvent>>::new();

		for event in self.events {
			match event {
				InternalEvent {
					data: InternalEventData::UserPresence(data),
					..
				} => {
					presence_data.push(*data);
				}
				event => {
					if let Some(event_type) = event.data.event_api_event_type() {
						events
							.entry((event_type, event.data.id(), event.data.event_api_kind()))
							.or_default()
							.push(event);
					}
				}
			};
		}

		let mut messages = vec![];

		for ((event_type, id, kind), events) in events {
			let mut updated = vec![];
			let mut pushed = vec![];
			let mut pulled = vec![];
			let mut versions_nested = vec![];
			let mut event_actor = None;

			for payload in events {
				event_actor = event_actor.or(payload.actor);

				match payload.data {
					InternalEventData::Emote {
						data: StoredEventEmoteData::ChangeName { old, new },
						..
					} => {
						let change = ChangeField {
							key: "name".to_string(),
							ty: ChangeFieldType::String,
							old_value: old.into(),
							value: new.into(),
							..Default::default()
						};
						updated.push(change.clone());
						versions_nested.push(change);
					}
					InternalEventData::Emote {
						data: StoredEventEmoteData::ChangeFlags { old, new },
						..
					} => {
						if old.contains(EmoteFlags::PublicListed) != new.contains(EmoteFlags::PublicListed) {
							let change = ChangeField {
								key: "listed".to_string(),
								ty: ChangeFieldType::Bool,
								old_value: old.contains(EmoteFlags::PublicListed).into(),
								value: new.contains(EmoteFlags::PublicListed).into(),
								..Default::default()
							};
							updated.push(change.clone());
							versions_nested.push(change);

							// convert to old model
							let old = EmoteFlagsModel::from(old);
							let new = EmoteFlagsModel::from(new);

							if old != new {
								updated.push(ChangeField {
									key: "flags".to_string(),
									ty: ChangeFieldType::Number,
									old_value: old.bits().into(),
									value: new.bits().into(),
									..Default::default()
								});
							}
						}
					}
					InternalEventData::Emote {
						data: StoredEventEmoteData::ChangeOwner { old, new },
						..
					} => {
						updated.push(ChangeField {
							key: "owner_id".to_string(),
							ty: ChangeFieldType::String,
							old_value: old.to_string().into(),
							value: new.to_string().into(),
							..Default::default()
						});
					}
					InternalEventData::Emote {
						data: StoredEventEmoteData::ChangeTags { old, new },
						..
					} => {
						updated.push(ChangeField {
							key: "tags".to_string(),
							old_value: old.into(),
							value: new.into(),
							..Default::default()
						});
					}
					InternalEventData::EmoteSet {
						data:
							InternalEventEmoteSetData::AddEmote {
								emote,
								emote_owner,
								emote_set_emote,
							},
						after,
					} => {
						let index = after
							.emotes
							.into_iter()
							.position(|e| e.id == emote.id)
							.context("failed to find emote in set")?;

						let owner = emote_owner.map(|u| UserPartialModel::from_db(*u, None, None, cdn_base_url));

						let active_emote = ActiveEmoteModel::from_db(
							emote_set_emote,
							Some(EmotePartialModel::from_db(*emote, owner, cdn_base_url)),
						);
						let active_emote = serde_json::to_value(active_emote)?;

						pushed.push(ChangeField {
							key: "emotes".to_string(),
							index: Some(index),
							ty: ChangeFieldType::Object,
							value: active_emote,
							..Default::default()
						});
					}
					InternalEventData::EmoteSet {
						data:
							InternalEventEmoteSetData::RemoveEmote {
								emote,
								emote_owner,
								emote_set_emote,
								index,
							},
						..
					} => {
						let owner = emote_owner.map(|u| UserPartialModel::from_db(*u, None, None, cdn_base_url));

						let active_emote = ActiveEmoteModel::from_db(
							emote_set_emote,
							emote.map(|emote| EmotePartialModel::from_db(*emote, owner, cdn_base_url)),
						);
						let active_emote = serde_json::to_value(active_emote)?;

						pulled.push(ChangeField {
							key: "emotes".to_string(),
							index: Some(index),
							ty: ChangeFieldType::Object,
							old_value: active_emote,
							..Default::default()
						});
					}
					InternalEventData::EmoteSet {
						data:
							InternalEventEmoteSetData::RenameEmote {
								emote,
								emote_set_emote,
								old_alias,
							},
						after,
					} => {
						let index = after
							.emotes
							.into_iter()
							.position(|e| e.id == emote.id)
							.context("failed to find emote in set")?;

						let new_active_emote = ActiveEmoteModel::from_db(
							emote_set_emote,
							Some(EmotePartialModel::from_db(*emote, None, cdn_base_url)),
						);

						let mut old_active_emote = new_active_emote.clone();
						old_active_emote.name = old_alias;

						let old_active_emote = serde_json::to_value(old_active_emote)?;
						let new_active_emote = serde_json::to_value(new_active_emote)?;

						updated.push(ChangeField {
							key: "emotes".to_string(),
							index: Some(index),
							ty: ChangeFieldType::Object,
							old_value: old_active_emote,
							value: new_active_emote,
							..Default::default()
						});
					}
					InternalEventData::EmoteSet {
						data: InternalEventEmoteSetData::ChangeName { old, new },
						..
					} => {
						updated.push(ChangeField {
							key: "name".to_string(),
							ty: ChangeFieldType::String,
							old_value: old.into(),
							value: new.into(),
							..Default::default()
						});
					}
					InternalEventData::EmoteSet {
						data: InternalEventEmoteSetData::ChangeCapacity { old, new },
						..
					} => {
						updated.push(ChangeField {
							key: "capacity".to_string(),
							ty: ChangeFieldType::Number,
							old_value: old.into(),
							value: new.into(),
							..Default::default()
						});
					}
					InternalEventData::EmoteSet {
						data: InternalEventEmoteSetData::Delete,
						..
					} => {
						// no updates
						// do not remove this match arm, this is a valid event
					}
					InternalEventData::User {
						after,
						data: InternalEventUserData::AddConnection { connection },
					} => {
						let index = after
							.connections
							.into_iter()
							.position(|c| c.platform_id == connection.platform_id)
							.context("failed to find connection")?;

						let value = serde_json::to_value(UserConnectionModel::from(UserConnectionPartialModel::from_db(
							connection,
							after.style.active_emote_set_id,
							0,
						)))?;
						// TODO: set to 0 for now, honestly we shouldnt care about this, nobody is
						// listening to this event anyway

						pushed.push(ChangeField {
							key: "connections".to_string(),
							ty: ChangeFieldType::Object,
							index: Some(index),
							value,
							..Default::default()
						});
					}
					InternalEventData::User {
						after,
						data: InternalEventUserData::RemoveConnection { connection },
					} => {
						let value = serde_json::to_value(UserConnectionModel::from(UserConnectionPartialModel::from_db(
							connection,
							after.style.active_emote_set_id,
							0,
						)))?;
						// TODO: set to 0 for now, honestly we shouldnt care about this, nobody is
						// listening to this event anyway This is all pointless, you can't even remove a
						// connection on the current website

						pulled.push(ChangeField {
							key: "connections".to_string(),
							ty: ChangeFieldType::Object,
							index: Some(after.connections.len()),
							value,
							..Default::default()
						});
					}
					InternalEventData::User {
						after,
						data: InternalEventUserData::ChangeActiveEmoteSet { old, new },
					} => {
						// we have to emit the event for every connection since you could have different
						// sets for every connection before

						let old_set = old.map(|set| EmoteSetModel::from_db(*set, std::iter::empty(), None));
						let new_set = new.map(|set| EmoteSetModel::from_db(*set, std::iter::empty(), None));

						for i in 0..after.connections.len() {
							let value = vec![
								ChangeField {
									key: "emote_set".to_string(),
									ty: ChangeFieldType::Object,
									old_value: serde_json::to_value(&old_set)?,
									value: serde_json::to_value(&new_set)?,
									..Default::default()
								},
								ChangeField {
									key: "emote_set_id".to_string(),
									ty: ChangeFieldType::String,
									old_value: old_set.as_ref().map(|s| s.id.to_string()).into(),
									value: new_set.as_ref().map(|s| s.id.to_string()).into(),
									..Default::default()
								},
							];

							let value = serde_json::to_value(value)?;

							updated.push(ChangeField {
								key: "connections".to_string(),
								index: Some(i),
								nested: true,
								value,
								..Default::default()
							});
						}
					}
					InternalEventData::User {
						data: InternalEventUserData::ChangeActivePaint { old, new },
						..
					} => {
						updated.push(ChangeField {
							key: "style".to_string(),
							ty: ChangeFieldType::Object,
							nested: true,
							value: serde_json::to_value([
								ChangeField {
									key: "paint".to_string(),
									ty: ChangeFieldType::Object,
									value: serde_json::to_value(&new)?,
									old_value: serde_json::to_value(&old)?,
									..Default::default()
								},
								ChangeField {
									key: "paint_id".to_string(),
									ty: ChangeFieldType::String,
									value: new.map(|p| p.id.to_string()).into(),
									old_value: old.map(|p| p.id.to_string()).into(),
									..Default::default()
								},
							])?,
							..Default::default()
						});
					}
					InternalEventData::User {
						data: InternalEventUserData::ChangeActiveBadge { old, new },
						..
					} => {
						updated.push(ChangeField {
							key: "style".to_string(),
							ty: ChangeFieldType::Object,
							nested: true,
							value: serde_json::to_value([
								ChangeField {
									key: "badge".to_string(),
									ty: ChangeFieldType::Object,
									value: serde_json::to_value(&new)?,
									old_value: serde_json::to_value(&old)?,
									..Default::default()
								},
								ChangeField {
									key: "badge_id".to_string(),
									ty: ChangeFieldType::String,
									value: new.map(|b| b.id.to_string()).into(),
									old_value: old.map(|b| b.id.to_string()).into(),
									..Default::default()
								},
							])?,
							..Default::default()
						});
					}
					_ => continue,
				}
			}

			if !versions_nested.is_empty() {
				let versions_nested = serde_json::to_value(versions_nested)?;

				updated.push(ChangeField {
					key: "versions".to_string(),
					nested: true,
					index: Some(0),
					value: versions_nested,
					..Default::default()
				});
			}

			let body = event_api::types::ChangeMap {
				id,
				actor: event_actor.map(|a| UserPartialModel::from_db(a, None, None, cdn_base_url)),
				kind,
				updated,
				pushed,
				pulled,
				..Default::default()
			};

			let dispatch = event_api::payload::Dispatch { ty: event_type, body };
			messages.push(event_api::Message::new(dispatch, seq));
		}

		Ok((messages, presence_data))
	}
}
