use postgres_types::{FromSql, ToSql};

use crate::database::Table;

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "ticket_priority")]
pub enum TicketPriority {
	#[default]
	#[postgres(name = "LOW")]
	Low,
	#[postgres(name = "MEDIUM")]
	Medium,
	#[postgres(name = "HIGH")]
	High,
	#[postgres(name = "URGENT")]
	Urgent,
}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "ticket_kind")]
pub enum TicketKind {
	#[postgres(name = "EMOTE_REPORT")]
	EmoteReport,
	#[postgres(name = "USER_REPORT")]
	UserReport,
	#[postgres(name = "BILLING")]
	Billing,
	#[postgres(name = "EMOTE_LISTING_REQUEST")]
	EmoteListingRequest,
	#[postgres(name = "EMOTE_PERSONAL_USE_REQUEST")]
	EmotePersonalUseRequest,
	#[default]
	#[postgres(name = "OTHER")]
	Other,
}

#[derive(Debug, Clone, Default, ToSql, FromSql, serde::Serialize, serde::Deserialize)]
#[postgres(name = "ticket_status")]
pub enum TicketStatus {
	#[default]
	#[postgres(name = "PENDING")]
	Pending,
	#[postgres(name = "IN_PROGRESS")]
	InProgress,
	#[postgres(name = "FIXED")]
	Fixed,
	#[postgres(name = "CLOSED")]
	Closed,
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct Ticket {
	pub id: ulid::Ulid,
	pub kind: TicketKind,
	pub status: TicketStatus,
	pub priority: TicketPriority,
	pub title: String,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: TicketData,
	pub tags: Vec<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Table for Ticket {
	const TABLE_NAME: &'static str = "tickets";
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct TicketData {}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct TicketFile {
	pub ticket_id: ulid::Ulid,
	pub file_id: ulid::Ulid,
}

impl Table for TicketFile {
	const TABLE_NAME: &'static str = "ticket_files";
}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "ticket_member_kind")]
pub enum TicketMemberKind {
	#[default]
	#[postgres(name = "OP")]
	Op,
	#[postgres(name = "MEMBER")]
	Member,
	#[postgres(name = "STAFF")]
	Staff,
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct TicketMember {
	pub ticket_id: ulid::Ulid,
	pub user_id: ulid::Ulid,
	pub kind: TicketMemberKind,
	pub notifications: bool,
}

impl Table for TicketMember {
	const TABLE_NAME: &'static str = "ticket_members";
}
