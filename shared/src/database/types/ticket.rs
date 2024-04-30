use super::{EmoteId, FileSetId, UserId};
use crate::database::{Collection, Id};

#[derive(Debug, Clone, Default, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum TicketPriority {
	#[default]
	Low = 0,
	Medium = 1,
	High = 2,
	Urgent = 3,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum TicketData {
	EmoteReport { emote_id: EmoteId },
	UserReport { user_id: UserId },
	Billing,
	EmoteListingRequest { emote_id: EmoteId },
	EmotePersonalUseRequest { emote_id: EmoteId },
	#[default]
	Other,
}

#[derive(Debug, Clone, Default, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum TicketStatus {
	#[default]
	Pending = 0,
	InProgress = 1,
	Fixed = 2,
	Closed = 3,
}

pub type TicketId = Id<Ticket>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ticket {
	#[serde(rename = "_id")]
	pub id: TicketId,
	pub status: TicketStatus,
	pub priority: TicketPriority,
	pub title: String,
	pub tags: Vec<String>,
	pub data: TicketData,
}

impl Collection for Ticket {
	const COLLECTION_NAME: &'static str = "tickets";
}

#[derive(Debug, Clone, Default, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum TicketMemberKind {
	#[default]
	Op = 0,
	Member = 1,
	Staff = 2,
}

pub type TicketMemberId = Id<TicketMember>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct TicketMember {
	#[serde(rename = "_id")]
	pub id: TicketMemberId,
	pub ticket_id: TicketId,
	pub user_id: UserId,
	pub kind: TicketMemberKind,
	pub notifications: bool,
}

impl Collection for TicketMember {
	const COLLECTION_NAME: &'static str = "ticket_members";
}
