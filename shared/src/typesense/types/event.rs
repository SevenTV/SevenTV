use chrono::Utc;

use super::{impl_typesense_type, TypesenseCollection, TypesenseGenericCollection};
use crate::database::badge::BadgeId;
use crate::database::emote::EmoteId;
use crate::database::emote_moderation_request::EmoteModerationRequestId;
use crate::database::emote_set::EmoteSetId;
use crate::database::entitlement::EntitlementEdgeKind;
use crate::database::paint::PaintId;
use crate::database::product::special_event::SpecialEventId;
use crate::database::product::ProductId;
use crate::database::role::RoleId;
use crate::database::stored_event::{
	ImageProcessorEvent, StoredEventBadgeData, StoredEventData, StoredEventEmoteData, StoredEventEmoteModerationRequestData,
	StoredEventEmoteSetData, StoredEventId, StoredEventPaintData, StoredEventRoleData, StoredEventTicketData,
	StoredEventTicketMessageData, StoredEventUserBanData, StoredEventUserData, StoredEventUserEditorData,
	StoredEventUserProfilePictureData, StoredEventUserSessionData,
};
use crate::database::ticket::{TicketId, TicketMessageId};
use crate::database::user::ban::UserBanId;
use crate::database::user::profile_picture::UserProfilePictureId;
use crate::database::user::session::UserSessionId;
use crate::database::user::UserId;
use crate::database::{self, IdFromStrError};

fn split_kinds(data: &StoredEventData) -> (EventId, ActionKind, Vec<EventId>) {
	match data {
		StoredEventData::Emote { target_id, data } => {
			let target = EventId::Emote(*target_id);
			let mut secondary = Vec::new();

			let action = match data {
				StoredEventEmoteData::Upload => ActionKind::EmoteUpload,
				StoredEventEmoteData::Process { event } => match event {
					ImageProcessorEvent::Success(_) => ActionKind::EmoteProcessSuccess,
					ImageProcessorEvent::Fail(_) => ActionKind::EmoteProcessFailure,
					ImageProcessorEvent::Cancel(_) => ActionKind::EmoteProcessCancel,
					ImageProcessorEvent::Start(_) => ActionKind::EmoteProcessStart,
				},
				StoredEventEmoteData::ChangeName { .. } => ActionKind::EmoteChangeName,
				StoredEventEmoteData::ChangeFlags { .. } => ActionKind::EmoteChangeFlags,
				StoredEventEmoteData::ChangeTags { .. } => ActionKind::EmoteChangeTags,
				StoredEventEmoteData::Merge { new_emote_id } => {
					secondary.push(EventId::Emote(*new_emote_id));
					ActionKind::EmoteMerge
				}
				StoredEventEmoteData::Delete => ActionKind::EmoteDelete,
				StoredEventEmoteData::ChangeOwner { old, new } => {
					secondary.push(EventId::User(*old));
					secondary.push(EventId::User(*new));
					ActionKind::EmoteChangeOwner
				}
			};

			(target, action, secondary)
		}
		StoredEventData::EmoteSet { target_id, data } => {
			let target = EventId::EmoteSet(*target_id);
			let mut secondary = Vec::new();

			let action = match data {
				StoredEventEmoteSetData::Create => ActionKind::EmoteSetCreate,
				StoredEventEmoteSetData::ChangeName { .. } => ActionKind::EmoteSetChangeName,
				StoredEventEmoteSetData::ChangeTags { .. } => ActionKind::EmoteSetChangeTags,
				StoredEventEmoteSetData::ChangeCapacity { .. } => ActionKind::EmoteSetChangeCapacity,
				StoredEventEmoteSetData::AddEmote { emote_id, .. } => {
					secondary.push(EventId::Emote(*emote_id));
					ActionKind::EmoteSetAddEmote
				}
				StoredEventEmoteSetData::RemoveEmote { emote_id } => {
					secondary.push(EventId::Emote(*emote_id));
					ActionKind::EmoteSetRemoveEmote
				}
				StoredEventEmoteSetData::RenameEmote { emote_id, .. } => {
					secondary.push(EventId::Emote(*emote_id));
					ActionKind::EmoteSetRenameEmote
				}
				StoredEventEmoteSetData::Delete => ActionKind::EmoteSetDelete,
			};

			(target, action, secondary)
		}
		StoredEventData::User { target_id, data } => {
			let target = EventId::User(*target_id);
			let mut secondary = Vec::new();

			let action = match data {
				StoredEventUserData::Create => ActionKind::UserCreate,
				StoredEventUserData::ChangeActivePaint { old, new } => {
					if let Some(id) = *old {
						secondary.push(EventId::Paint(id))
					}
					if let Some(id) = *new {
						secondary.push(EventId::Paint(id))
					}
					ActionKind::UserChangeActivePaint
				}
				StoredEventUserData::ChangeActiveBadge { old, new } => {
					if let Some(id) = *old {
						secondary.push(EventId::Badge(id))
					}
					if let Some(id) = *new {
						secondary.push(EventId::Badge(id))
					}
					ActionKind::UserChangeActiveBadge
				}
				StoredEventUserData::ChangeActiveEmoteSet { old, new } => {
					if let Some(id) = *old {
						secondary.push(EventId::EmoteSet(id))
					}
					if let Some(id) = *new {
						secondary.push(EventId::EmoteSet(id))
					}
					ActionKind::UserChangeActiveEmoteSet
				}
				StoredEventUserData::AddConnection { .. } => ActionKind::UserAddConnection,
				StoredEventUserData::RemoveConnection { .. } => ActionKind::UserRemoveConnection,
				StoredEventUserData::Merge => ActionKind::UserMerge,
				StoredEventUserData::Delete => ActionKind::UserDelete,
				StoredEventUserData::AddEntitlement { target } => {
					match target {
						EntitlementEdgeKind::Role { role_id } => secondary.push(EventId::Role(*role_id)),
						EntitlementEdgeKind::Badge { badge_id } => secondary.push(EventId::Badge(*badge_id)),
						EntitlementEdgeKind::Paint { paint_id } => secondary.push(EventId::Paint(*paint_id)),
						EntitlementEdgeKind::EmoteSet { emote_set_id: emote_id } => {
							secondary.push(EventId::EmoteSet(*emote_id))
						}
						EntitlementEdgeKind::Product { product_id } => secondary.push(EventId::Product(product_id.clone())),
						EntitlementEdgeKind::SpecialEvent { special_event_id } => {
							secondary.push(EventId::SpecialEvent(*special_event_id))
						}
						EntitlementEdgeKind::Subscription { .. } => {}
						EntitlementEdgeKind::SubscriptionBenefit { .. } => {}
						EntitlementEdgeKind::GlobalDefaultEntitlementGroup => {}
						EntitlementEdgeKind::User { .. } => {}
					}

					ActionKind::UserAddEntitlement
				}
				StoredEventUserData::RemoveEntitlement { target } => {
					match target {
						EntitlementEdgeKind::Role { role_id } => secondary.push(EventId::Role(*role_id)),
						EntitlementEdgeKind::Badge { badge_id } => secondary.push(EventId::Badge(*badge_id)),
						EntitlementEdgeKind::Paint { paint_id } => secondary.push(EventId::Paint(*paint_id)),
						EntitlementEdgeKind::EmoteSet { emote_set_id: emote_id } => {
							secondary.push(EventId::EmoteSet(*emote_id))
						}
						EntitlementEdgeKind::Product { product_id } => secondary.push(EventId::Product(product_id.clone())),
						EntitlementEdgeKind::SpecialEvent { special_event_id } => {
							secondary.push(EventId::SpecialEvent(*special_event_id))
						}
						EntitlementEdgeKind::Subscription { .. } => {}
						EntitlementEdgeKind::SubscriptionBenefit { .. } => {}
						EntitlementEdgeKind::GlobalDefaultEntitlementGroup => {}
						EntitlementEdgeKind::User { .. } => {}
					}

					ActionKind::UserRemoveEntitlement
				}
			};

			(target, action, secondary)
		}
		StoredEventData::UserEditor { target_id, data } => {
			let target = EventId::User(target_id.user_id);
			let secondary = vec![EventId::User(target_id.editor_id)];

			let action = match data {
				StoredEventUserEditorData::AddEditor { .. } => ActionKind::UserEditorAdd,
				StoredEventUserEditorData::RemoveEditor { .. } => ActionKind::UserEditorRemove,
				StoredEventUserEditorData::EditPermissions { .. } => ActionKind::UserEditorEditPermissions,
			};

			(target, action, secondary)
		}
		StoredEventData::UserBan {
			target_id,
			user_id,
			data,
		} => {
			let target = EventId::UserBan(*target_id);
			let secondary = vec![EventId::User(*user_id)];

			let action = match data {
				StoredEventUserBanData::Ban => ActionKind::UserBanCreate,
				StoredEventUserBanData::ChangeReason { .. } => ActionKind::UserBanChangeReason,
				StoredEventUserBanData::ChangeExpiresAt { .. } => ActionKind::UserBanChangeExpiresAt,
				StoredEventUserBanData::ChangeUserBanPermissions { .. } => ActionKind::UserBanChangeUserBanPermissions,
				StoredEventUserBanData::Unban => ActionKind::UserBanUnban,
			};

			(target, action, secondary)
		}
		StoredEventData::UserSession {
			target_id,
			user_id,
			data,
		} => {
			let target = EventId::UserSession(*target_id);
			let secondary = vec![EventId::User(*user_id)];

			let action = match data {
				StoredEventUserSessionData::Create { .. } => ActionKind::UserSessionCreate,
				StoredEventUserSessionData::Delete => ActionKind::UserSessionDelete,
			};

			(target, action, secondary)
		}
		StoredEventData::Ticket { target_id, data } => {
			let target = EventId::Ticket(*target_id);
			let mut secondary = Vec::new();

			let action = match data {
				StoredEventTicketData::Create => ActionKind::TicketCreate,
				StoredEventTicketData::AddMember { member } => {
					secondary.push(EventId::User(*member));
					ActionKind::TicketAddMember
				}
				StoredEventTicketData::RemoveMember { member } => {
					secondary.push(EventId::User(*member));
					ActionKind::TicketRemoveMember
				}
				StoredEventTicketData::ChangeOpen { .. } => ActionKind::TicketChangeOpen,
				StoredEventTicketData::ChangePriority { .. } => ActionKind::TicketChangePriority,
			};

			(target, action, secondary)
		}
		StoredEventData::TicketMessage {
			target_id,
			data,
			ticket_id,
		} => {
			let target = EventId::TicketMessage(*target_id);
			let secondary = vec![EventId::Ticket(*ticket_id)];

			let action = match data {
				StoredEventTicketMessageData::Create => ActionKind::TicketMessageCreate,
			};

			(target, action, secondary)
		}
		StoredEventData::UserProfilePicture {
			target_id,
			data,
			user_id,
		} => {
			let target = EventId::UserProfilePicture(*target_id);
			let secondary = vec![EventId::User(*user_id)];

			let action = match data {
				StoredEventUserProfilePictureData::Create => ActionKind::UserProfilePictureCreate,
				StoredEventUserProfilePictureData::Process { event } => match event {
					ImageProcessorEvent::Success(_) => ActionKind::UserProfilePictureProcessSuccess,
					ImageProcessorEvent::Fail(_) => ActionKind::UserProfilePictureProcessFailure,
					ImageProcessorEvent::Cancel(_) => ActionKind::UserProfilePictureProcessCancel,
					ImageProcessorEvent::Start(_) => ActionKind::UserProfilePictureProcessStart,
				},
			};

			(target, action, secondary)
		}
		StoredEventData::EmoteModerationRequest {
			target_id,
			emote_id,
			data,
		} => {
			let target = EventId::EmoteModerationRequest(*target_id);
			let secondary = vec![EventId::Emote(*emote_id)];

			let action = match data {
				StoredEventEmoteModerationRequestData::Create => ActionKind::EmoteModerationRequestCreate,
			};

			(target, action, secondary)
		}
		StoredEventData::Paint { target_id, data } => {
			let target = EventId::Paint(*target_id);
			let secondary = Vec::new();

			let action = match data {
				StoredEventPaintData::Create => ActionKind::PaintCreate,
				StoredEventPaintData::ChangeName { .. } => ActionKind::PaintChangeName,
				StoredEventPaintData::ChangeData { .. } => ActionKind::PaintChangeData,
				StoredEventPaintData::Process { event } => match event {
					ImageProcessorEvent::Success(_) => ActionKind::PaintProcessSuccess,
					ImageProcessorEvent::Fail(_) => ActionKind::PaintProcessFailure,
					ImageProcessorEvent::Cancel(_) => ActionKind::PaintProcessCancel,
					ImageProcessorEvent::Start(_) => ActionKind::PaintProcessStart,
				},
			};

			(target, action, secondary)
		}
		StoredEventData::Badge { target_id, data } => {
			let target = EventId::Badge(*target_id);
			let secondary = Vec::new();

			let action = match data {
				StoredEventBadgeData::Create => ActionKind::BadgeCreate,
				StoredEventBadgeData::Process { event } => match event {
					ImageProcessorEvent::Success(_) => ActionKind::BadgeProcessSuccess,
					ImageProcessorEvent::Fail(_) => ActionKind::BadgeProcessFailure,
					ImageProcessorEvent::Cancel(_) => ActionKind::BadgeProcessCancel,
					ImageProcessorEvent::Start(_) => ActionKind::BadgeProcessStart,
				},
			};

			(target, action, secondary)
		}
		StoredEventData::Role { target_id, data } => {
			let target = EventId::Role(*target_id);
			let mut secondary = Vec::new();

			let action = match data {
				StoredEventRoleData::Create => ActionKind::RoleCreate,
				StoredEventRoleData::ChangeName { .. } => ActionKind::RoleChangeName,
				StoredEventRoleData::ChangeColor { .. } => ActionKind::RoleChangeColor,
				StoredEventRoleData::ChangePermissions { .. } => ActionKind::RoleChangePermissions,
				StoredEventRoleData::ChangeRank { .. } => ActionKind::RoleChangeRank,
				StoredEventRoleData::AddEntitlement { target } => {
					match target {
						EntitlementEdgeKind::Role { role_id } => secondary.push(EventId::Role(*role_id)),
						EntitlementEdgeKind::Badge { badge_id } => secondary.push(EventId::Badge(*badge_id)),
						EntitlementEdgeKind::Paint { paint_id } => secondary.push(EventId::Paint(*paint_id)),
						EntitlementEdgeKind::EmoteSet { emote_set_id: emote_id } => {
							secondary.push(EventId::EmoteSet(*emote_id))
						}
						EntitlementEdgeKind::Product { product_id } => secondary.push(EventId::Product(product_id.clone())),
						EntitlementEdgeKind::SpecialEvent { special_event_id } => {
							secondary.push(EventId::SpecialEvent(*special_event_id))
						}
						EntitlementEdgeKind::Subscription { .. } => {}
						EntitlementEdgeKind::SubscriptionBenefit { .. } => {}
						EntitlementEdgeKind::GlobalDefaultEntitlementGroup => {}
						EntitlementEdgeKind::User { .. } => {}
					}

					ActionKind::RoleAddEntitlement
				}
				StoredEventRoleData::RemoveEntitlement { target } => {
					match target {
						EntitlementEdgeKind::Role { role_id } => secondary.push(EventId::Role(*role_id)),
						EntitlementEdgeKind::Badge { badge_id } => secondary.push(EventId::Badge(*badge_id)),
						EntitlementEdgeKind::Paint { paint_id } => secondary.push(EventId::Paint(*paint_id)),
						EntitlementEdgeKind::EmoteSet { emote_set_id: emote_id } => {
							secondary.push(EventId::EmoteSet(*emote_id))
						}
						EntitlementEdgeKind::Product { product_id } => secondary.push(EventId::Product(product_id.clone())),
						EntitlementEdgeKind::SpecialEvent { special_event_id } => {
							secondary.push(EventId::SpecialEvent(*special_event_id))
						}
						EntitlementEdgeKind::Subscription { .. } => {}
						EntitlementEdgeKind::SubscriptionBenefit { .. } => {}
						EntitlementEdgeKind::GlobalDefaultEntitlementGroup => {}
						EntitlementEdgeKind::User { .. } => {}
					}

					ActionKind::RoleRemoveEntitlement
				}
				StoredEventRoleData::Delete => ActionKind::RoleDelete,
			};

			(target, action, secondary)
		}
	}
}

#[derive(Debug, Clone)]
pub enum EventId {
	User(UserId),
	Emote(EmoteId),
	EmoteSet(EmoteSetId),
	Badge(BadgeId),
	Paint(PaintId),
	Role(RoleId),
	Product(ProductId),
	SpecialEvent(SpecialEventId),
	UserProfilePicture(UserProfilePictureId),
	UserBan(UserBanId),
	UserSession(UserSessionId),
	Ticket(TicketId),
	TicketMessage(TicketMessageId),
	EmoteModerationRequest(EmoteModerationRequestId),
}

impl std::fmt::Display for EventId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			EventId::User(user_id) => write!(f, "user:{}", user_id),
			EventId::Emote(emote_id) => write!(f, "emote:{}", emote_id),
			EventId::EmoteSet(emote_set_id) => write!(f, "emote_set:{}", emote_set_id),
			EventId::Badge(badge_id) => write!(f, "badge:{}", badge_id),
			EventId::Paint(paint_id) => write!(f, "paint:{}", paint_id),
			EventId::Role(role_id) => write!(f, "role:{}", role_id),
			EventId::Product(product_id) => write!(f, "product:{}", product_id),
			EventId::SpecialEvent(special_event_id) => write!(f, "special_event:{}", special_event_id),
			EventId::UserProfilePicture(user_profile_picture_id) => {
				write!(f, "user_profile_picture:{}", user_profile_picture_id)
			}
			EventId::UserBan(user_ban_id) => write!(f, "user_ban:{}", user_ban_id),
			EventId::UserSession(user_session_id) => write!(f, "user_session:{}", user_session_id),
			EventId::Ticket(ticket_id) => write!(f, "ticket:{}", ticket_id),
			EventId::TicketMessage(ticket_message_id) => write!(f, "ticket_message:{}", ticket_message_id),
			EventId::EmoteModerationRequest(emote_moderation_request_id) => {
				write!(f, "emote_moderation_request:{}", emote_moderation_request_id)
			}
		}
	}
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum EventIdFromStrError {
	#[error("invalid event id format")]
	BadFormat,
	#[error("unknown event id kind: {0}")]
	UnknownKind(String),
	#[error("invalid id: {0}")]
	InvalidId(#[from] IdFromStrError),
	#[error("invalid id: {0}")]
	OtherInvalidId(&'static str),
}

impl std::str::FromStr for EventId {
	type Err = EventIdFromStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (kind, id) = s.split_once(':').ok_or(EventIdFromStrError::BadFormat)?;

		Ok(match kind {
			"user" => EventId::User(id.parse()?),
			"emote" => EventId::Emote(id.parse()?),
			"emote_set" => EventId::EmoteSet(id.parse()?),
			"badge" => EventId::Badge(id.parse()?),
			"paint" => EventId::Paint(id.parse()?),
			"role" => EventId::Role(id.parse()?),
			"product" => EventId::Product(id.parse().map_err(|_| EventIdFromStrError::OtherInvalidId("product"))?),
			"special_event" => EventId::SpecialEvent(id.parse()?),
			"user_profile_picture" => EventId::UserProfilePicture(id.parse()?),
			"user_ban" => EventId::UserBan(id.parse()?),
			"user_session" => EventId::UserSession(id.parse()?),
			"ticket" => EventId::Ticket(id.parse()?),
			"ticket_message" => EventId::TicketMessage(id.parse()?),
			"emote_moderation_request" => EventId::EmoteModerationRequest(id.parse()?),
			_ => return Err(EventIdFromStrError::UnknownKind(kind.to_string())),
		})
	}
}

impl serde::Serialize for EventId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

impl<'de> serde::Deserialize<'de> for EventId {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		s.parse().map_err(serde::de::Error::custom)
	}
}

impl_typesense_type!(EventId, String);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypesenseCollection)]
#[typesense(collection_name = "events")]
#[serde(deny_unknown_fields)]
pub struct Event {
	pub id: StoredEventId,
	pub actor_id: Option<UserId>,
	pub target_id: EventId,
	pub related_ids: Vec<EventId>,
	pub session_id: Option<UserSessionId>,
	pub action: ActionKind,
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::stored_event::StoredEvent> for Event {
	fn from(value: database::stored_event::StoredEvent) -> Self {
		let (target_id, action, related_ids) = split_kinds(&value.data);

		Self {
			id: value.id,
			actor_id: value.actor_id,
			target_id,
			action,
			related_ids,
			session_id: value.session_id,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum ActionKind {
	EmoteUpload = 0,
	EmoteChangeName = 1,
	EmoteMerge = 2,
	EmoteChangeOwner = 3,
	EmoteChangeTags = 4,
	EmoteChangeFlags = 5,
	EmoteDelete = 6,
	EmoteProcessSuccess = 7,
	EmoteProcessFailure = 8,
	EmoteProcessCancel = 9,
	EmoteProcessStart = 10,

	EmoteSetCreate = 100,
	EmoteSetChangeName = 101,
	EmoteSetChangeTags = 102,
	EmoteSetChangeCapacity = 103,
	EmoteSetAddEmote = 104,
	EmoteSetRemoveEmote = 105,
	EmoteSetRenameEmote = 106,
	EmoteSetDelete = 107,

	UserCreate = 200,
	UserChangeActivePaint = 201,
	UserChangeActiveBadge = 202,
	UserChangeActiveEmoteSet = 203,
	UserAddConnection = 204,
	UserRemoveConnection = 205,
	UserMerge = 206,
	UserDelete = 207,
	UserAddEntitlement = 208,
	UserRemoveEntitlement = 209,

	UserProfilePictureCreate = 300,
	UserProfilePictureProcessSuccess = 301,
	UserProfilePictureProcessFailure = 302,
	UserProfilePictureProcessCancel = 303,
	UserProfilePictureProcessStart = 304,

	UserEditorAdd = 400,
	UserEditorRemove = 401,
	UserEditorEditPermissions = 402,

	UserBanCreate = 500,
	UserBanChangeReason = 501,
	UserBanChangeExpiresAt = 502,
	UserBanChangeUserBanPermissions = 503,
	UserBanUnban = 504,

	UserSessionCreate = 600,
	UserSessionDelete = 601,

	TicketCreate = 700,
	TicketAddMember = 701,
	TicketRemoveMember = 702,
	TicketChangeOpen = 703,
	TicketChangePriority = 704,

	TicketMessageCreate = 800,

	EmoteModerationRequestCreate = 900,

	PaintCreate = 1000,
	PaintChangeName = 1001,
	PaintChangeData = 1002,
	PaintProcessSuccess = 1003,
	PaintProcessFailure = 1004,
	PaintProcessCancel = 1005,
	PaintProcessStart = 1006,

	BadgeCreate = 1100,
	BadgeProcessSuccess = 1101,
	BadgeProcessFailure = 1102,
	BadgeProcessCancel = 1103,
	BadgeProcessStart = 1104,

	RoleCreate = 1200,
	RoleChangeName = 1201,
	RoleChangeColor = 1202,
	RoleChangePermissions = 1203,
	RoleChangeRank = 1204,
	RoleDelete = 1205,
	RoleAddEntitlement = 1206,
	RoleRemoveEntitlement = 1207,
}

impl_typesense_type!(ActionKind, Int32);

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<Event>()]
}
