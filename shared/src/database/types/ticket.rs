use crate::database::Table;

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
pub struct Ticket {
	pub id: ulid::Ulid,
	pub kind: TicketKind,
	pub status: TicketStatus,
	pub priority: TicketPriority,
	pub title: String,
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

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct TicketFile {
	pub ticket_id: ulid::Ulid,
	pub file_id: ulid::Ulid,
}

impl Table for TicketFile {
	const TABLE_NAME: &'static str = "ticket_files";
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum TicketMemberKind {
	#[default]
	Op,
	Member,
	Staff,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct TicketMember {
	pub ticket_id: ulid::Ulid,
	pub user_id: ulid::Ulid,
	pub kind: TicketMemberKind,
	pub notifications: bool,
}

impl Table for TicketMember {
	const TABLE_NAME: &'static str = "ticket_members";
}
