use super::{impl_typesense_type, TypesenseGenericCollection};
use crate::database::ticket::{TicketId, TicketKind, TicketMemberKind, TicketMessageId, TicketPriority, TicketTarget};
use crate::database::user::UserId;
use crate::typesense::types::TypesenseCollection;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "tickets")]
#[serde(deny_unknown_fields)]
pub struct Ticket {
	pub id: TicketId,
	#[typesense(default_sort)]
	pub priority: TicketPriority,
	pub members: Vec<UserId>,
	pub assignees: Vec<UserId>,
	pub watchers: Vec<UserId>,
	pub title: String,
	pub tags: Vec<String>,
	pub country_code: Option<String>,
	pub kind: TicketKind,
	pub targets: Vec<TicketTargetString>,
	pub targets_emote: bool,
	pub author_id: UserId,
	pub open: bool,
	pub locked: bool,
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<crate::database::ticket::Ticket> for Ticket {
	fn from(value: crate::database::ticket::Ticket) -> Self {
		Self {
			id: value.id,
			priority: value.priority,
			members: value
				.members
				.iter()
				.filter(|member| member.kind == TicketMemberKind::Member)
				.map(|member| member.user_id)
				.collect(),
			assignees: value
				.members
				.iter()
				.filter(|member| member.kind == TicketMemberKind::Assigned)
				.map(|member| member.user_id)
				.collect(),
			watchers: value
				.members
				.iter()
				.filter(|member| member.kind == TicketMemberKind::Watcher)
				.map(|member| member.user_id)
				.collect(),
			title: value.title,
			tags: value.tags,
			country_code: value.country_code,
			kind: value.kind,
			targets_emote: value.targets.iter().any(|t| matches!(t, TicketTarget::Emote(_))),
			targets: value.targets.into_iter().map(Into::into).collect(),
			author_id: value.author_id,
			open: value.open,
			locked: value.locked,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: chrono::Utc::now().timestamp_millis(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct TicketTargetString(pub TicketTarget);

impl From<TicketTarget> for TicketTargetString {
	fn from(value: TicketTarget) -> Self {
		Self(value)
	}
}

impl From<TicketTargetString> for TicketTarget {
	fn from(value: TicketTargetString) -> Self {
		value.0
	}
}

impl std::fmt::Display for TicketTargetString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl serde::Serialize for TicketTargetString {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.0.to_string())
	}
}

impl<'de> serde::Deserialize<'de> for TicketTargetString {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		Ok(Self(s.parse().map_err(serde::de::Error::custom)?))
	}
}

impl_typesense_type!(TicketTargetString, String);

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "ticket_messages")]
#[serde(deny_unknown_fields)]
pub struct TicketMessage {
	pub id: TicketMessageId,
	pub ticket_id: TicketId,
	pub user_id: UserId,
	pub content: String,
	pub has_attachment: bool,
}

impl From<crate::database::ticket::TicketMessage> for TicketMessage {
	fn from(value: crate::database::ticket::TicketMessage) -> Self {
		Self {
			id: value.id,
			ticket_id: value.ticket_id,
			user_id: value.user_id,
			content: value.content,
			has_attachment: !value.files.is_empty(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[
		TypesenseGenericCollection::new::<Ticket>(),
		TypesenseGenericCollection::new::<TicketMessage>(),
	]
}
