use super::emote::EmoteId;
use super::emote_set::EmoteSetId;
use super::user::UserId;
use super::MongoGenericCollection;
use crate::database::{Id, MongoCollection};
use crate::typesense::types::impl_typesense_type;

#[derive(Debug, Clone, Default, serde_repr::Deserialize_repr, serde_repr::Serialize_repr, PartialEq, Eq)]
#[repr(i32)]
pub enum TicketPriority {
	Low = 0,
	#[default]
	Medium = 1,
	High = 2,
	Urgent = 3,
}

impl TryFrom<i32> for TicketPriority {
	type Error = ();

	fn try_from(value: i32) -> Result<Self, Self::Error> {
		match value {
			0 => Ok(Self::Low),
			1 => Ok(Self::Medium),
			2 => Ok(Self::High),
			3 => Ok(Self::Urgent),
			_ => Err(()),
		}
	}
}

impl_typesense_type!(TicketPriority, Int32);

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum TicketTarget {
	User(UserId),
	Emote(EmoteId),
	EmoteSet(EmoteSetId),
}

impl std::fmt::Display for TicketTarget {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TicketTarget::Emote(id) => write!(f, "emote:{}", id),
			TicketTarget::EmoteSet(id) => write!(f, "emote_set:{}", id),
			TicketTarget::User(id) => write!(f, "user:{}", id),
		}
	}
}

impl std::str::FromStr for TicketTarget {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.split(':').collect::<Vec<_>>()[..] {
			["emote", id] => Ok(TicketTarget::Emote(id.parse().map_err(|_| "invalid emote id")?)),
			["emote_set", id] => Ok(TicketTarget::EmoteSet(id.parse().map_err(|_| "invalid emote set id")?)),
			["user", id] => Ok(TicketTarget::User(id.parse().map_err(|_| "invalid user id")?)),
			_ => Err("invalid target"),
		}
	}
}

#[derive(Debug, Clone, serde_repr::Deserialize_repr, serde_repr::Serialize_repr, PartialEq, Eq)]
#[repr(i32)]
pub enum TicketKind {
	Abuse = 0,
	Billing = 1,
	Generic = 2,
}

impl_typesense_type!(TicketKind, Int32);

pub type TicketId = Id<Ticket>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection, PartialEq, Eq)]
#[mongo(collection_name = "tickets")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(search = "crate::typesense::types::ticket::Ticket")]
#[serde(deny_unknown_fields)]
pub struct Ticket {
	#[mongo(id)]
	#[serde(rename = "_id")]
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
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(i32)]
pub enum TicketMemberKind {
	#[default]
	Member = 0,
	Assigned = 1,
	Watcher = 2,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TicketMember {
	pub user_id: UserId,
	pub kind: TicketMemberKind,
	pub notifications: bool,
	pub last_read: Option<TicketMessageId>,
}

pub type TicketMessageId = Id<TicketMessage>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, MongoCollection, PartialEq, Eq)]
#[mongo(collection_name = "ticket_messages")]
#[mongo(index(fields(ticket_id = 1)))]
#[mongo(index(fields(user_id = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(search = "crate::typesense::types::ticket::TicketMessage")]
#[serde(deny_unknown_fields)]
pub struct TicketMessage {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: TicketMessageId,
	pub ticket_id: TicketId,
	pub user_id: UserId,
	pub content: String,
	pub files: Vec<TicketFile>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TicketFile {
	pub path: String,
	pub mime: String,
	pub size: i64,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[
		MongoGenericCollection::new::<Ticket>(),
		MongoGenericCollection::new::<TicketMessage>(),
	]
}
