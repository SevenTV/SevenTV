use bson::oid::ObjectId;

use crate::database::Collection;

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

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ticket {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub kind: TicketKind,
	pub status: TicketStatus,
	pub priority: TicketPriority,
	pub title: String,
	pub tags: Vec<String>,
	pub files: Vec<ObjectId>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Collection for Ticket {
	const NAME: &'static str = "tickets";
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum TicketMemberKind {
	#[default]
	Op,
	Member,
	Staff,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct TicketMember {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub ticket_id: ObjectId,
	pub user_id: ObjectId,
	pub kind: TicketMemberKind,
	pub notifications: bool,
}

impl Collection for TicketMember {
	const NAME: &'static str = "ticket_members";
}
