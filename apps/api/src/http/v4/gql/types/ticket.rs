use shared::database::ticket::{TicketId, TicketMessageId};
use shared::database::user::UserId;
use shared::database::Id;

#[derive(Debug, Clone, async_graphql::SimpleObject)]
pub struct TicketTarget {
	kind: TicketTargetType,
	id: Id<()>,
}

impl From<shared::database::ticket::TicketTarget> for TicketTarget {
	fn from(value: shared::database::ticket::TicketTarget) -> Self {
		match value {
			shared::database::ticket::TicketTarget::User(id) => Self {
				kind: TicketTargetType::User,
				id: id.cast(),
			},
			shared::database::ticket::TicketTarget::Emote(id) => Self {
				kind: TicketTargetType::Emote,
				id: id.cast(),
			},
			shared::database::ticket::TicketTarget::EmoteSet(id) => Self {
				kind: TicketTargetType::EmoteSet,
				id: id.cast(),
			},
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, async_graphql::Enum)]
pub enum TicketTargetType {
	User,
	Emote,
	EmoteSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, async_graphql::Enum)]
pub enum TicketPriority {
	Low,
	Medium,
	High,
	Urgent,
}

impl From<shared::database::ticket::TicketPriority> for TicketPriority {
	fn from(value: shared::database::ticket::TicketPriority) -> Self {
		match value {
			shared::database::ticket::TicketPriority::Low => Self::Low,
			shared::database::ticket::TicketPriority::Medium => Self::Medium,
			shared::database::ticket::TicketPriority::High => Self::High,
			shared::database::ticket::TicketPriority::Urgent => Self::Urgent,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, async_graphql::Enum)]
pub enum TicketKind {
	Abuse,
	Billing,
	Generic,
}

impl From<shared::database::ticket::TicketKind> for TicketKind {
	fn from(value: shared::database::ticket::TicketKind) -> Self {
		match value {
			shared::database::ticket::TicketKind::Abuse => Self::Abuse,
			shared::database::ticket::TicketKind::Billing => Self::Billing,
			shared::database::ticket::TicketKind::Generic => Self::Generic,
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct Ticket {
	pub id: TicketId,
	pub priority: TicketPriority,
	pub members: Vec<TicketMember>,
	pub title: String,
	pub tags: Vec<String>,
	pub country_code: Option<String>,
	pub kind: TicketKind,
	pub targets: Vec<TicketTarget>,
	pub author_id: UserId,
	pub open: bool,
	pub locked: bool,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<shared::database::ticket::Ticket> for Ticket {
	fn from(value: shared::database::ticket::Ticket) -> Self {
		Self {
			id: value.id,
			priority: value.priority.into(),
			members: value.members.into_iter().map(Into::into).collect(),
			title: value.title,
			tags: value.tags,
			country_code: value.country_code,
			kind: value.kind.into(),
			targets: value.targets.into_iter().map(Into::into).collect(),
			author_id: value.author_id,
			open: value.open,
			locked: value.locked,
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, async_graphql::Enum)]
pub enum TicketMemberKind {
	Member,
	Assigned,
	Watcher,
}

impl From<shared::database::ticket::TicketMemberKind> for TicketMemberKind {
	fn from(value: shared::database::ticket::TicketMemberKind) -> Self {
		match value {
			shared::database::ticket::TicketMemberKind::Member => Self::Member,
			shared::database::ticket::TicketMemberKind::Assigned => Self::Assigned,
			shared::database::ticket::TicketMemberKind::Watcher => Self::Watcher,
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct TicketMember {
	pub user_id: UserId,
	pub kind: TicketMemberKind,
	pub notifications: bool,
	pub last_read: Option<TicketMessageId>,
}

impl From<shared::database::ticket::TicketMember> for TicketMember {
	fn from(value: shared::database::ticket::TicketMember) -> Self {
		Self {
			user_id: value.user_id,
			kind: value.kind.into(),
			notifications: value.notifications,
			last_read: value.last_read,
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct TicketMessage {
	pub id: TicketMessageId,
	pub ticket_id: TicketId,
	pub user_id: UserId,
	pub content: String,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<shared::database::ticket::TicketMessage> for TicketMessage {
	fn from(value: shared::database::ticket::TicketMessage) -> Self {
		Self {
			id: value.id,
			ticket_id: value.ticket_id,
			user_id: value.user_id,
			content: value.content,
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		}
	}
}
