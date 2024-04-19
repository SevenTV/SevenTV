use super::{FileSetId, UserId};
use crate::database::{Collection, Id};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum TicketPriority {
	#[default]
	Low,
	Medium,
	High,
	Urgent,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum TicketKind {
	EmoteReport,
	UserReport,
	Billing,
	EmoteListingRequest,
	EmotePersonalUseRequest,
	#[default]
	Other,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum TicketStatus {
	#[default]
	Pending,
	InProgress,
	Fixed,
	Closed,
}

pub type TicketId = Id<Ticket>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ticket {
	#[serde(rename = "_id", skip_serializing_if = "Id::is_nil")]
	pub id: TicketId,
	pub kind: TicketKind,
	pub status: TicketStatus,
	pub priority: TicketPriority,
	pub title: String,
	pub tags: Vec<String>,
	pub files: Vec<FileSetId>,
}

impl Collection for Ticket {
	const COLLECTION_NAME: &'static str = "tickets";
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum TicketMemberKind {
	#[default]
	Op,
	Member,
	Staff,
}

pub type TicketMemberId = Id<TicketMember>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct TicketMember {
	#[serde(rename = "_id", skip_serializing_if = "Id::is_nil")]
	pub id: TicketMemberId,
	pub ticket_id: TicketId,
	pub user_id: UserId,
	pub kind: TicketMemberKind,
	pub notifications: bool,
}

impl Collection for TicketMember {
	const COLLECTION_NAME: &'static str = "ticket_members";
}
