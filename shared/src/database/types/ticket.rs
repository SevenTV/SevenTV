use derive_builder::Builder;

use super::emote::EmoteId;
use super::emote_set::EmoteSetId;
use super::product::InvoiceId;
use super::user::UserId;
use super::GenericCollection;
use crate::database::{Collection, Id};

#[derive(Debug, Clone, Default, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(i32)]
pub enum TicketPriority {
	Low = 0,
	#[default]
	Medium = 1,
	High = 2,
	Urgent = 3,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum TicketTarget {
	User(UserId),
	Emote(EmoteId),
	EmoteSet(EmoteSetId),
	Invoice(InvoiceId),
}

#[derive(Debug, Clone, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(i32)]
pub enum TicketKind {
	Abuse = 0,
	Billing = 1,
	Generic = 2,
}

pub type TicketId = Id<Ticket>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct Ticket {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: TicketId,
	#[builder(default)]
	pub priority: TicketPriority,
	pub members: Vec<TicketMember>,
	pub title: String,
	#[builder(default)]
	pub tags: Vec<String>,
	#[builder(default)]
	pub country_code: Option<String>,
	pub kind: TicketKind,
	#[builder(default)]
	pub targets: Vec<TicketTarget>,
	pub author_id: UserId,
	#[builder(default = "true")]
	pub open: bool,
	#[builder(default = "false")]
	pub locked: bool,
}

impl Collection for Ticket {
	const COLLECTION_NAME: &'static str = "tickets";
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(i32)]
pub enum TicketMemberKind {
	#[default]
	Member = 0,
	Assigned = 1,
	Watcher = 2,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct TicketMember {
	pub user_id: UserId,
	#[builder(default)]
	pub kind: TicketMemberKind,
	#[builder(default = "true")]
	pub notifications: bool,
	#[builder(default)]
	pub last_read: Option<TicketMessageId>,
}

pub type TicketMessageId = Id<TicketMessage>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct TicketMessage {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: TicketMessageId,
	pub ticket_id: TicketId,
	pub user_id: UserId,
	pub content: String,
	#[builder(default)]
	pub files: Vec<TicketFile>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct TicketFile {
	pub path: String,
	pub mime: String,
	pub size: i64,
}

impl Collection for TicketMessage {
	const COLLECTION_NAME: &'static str = "ticket_messages";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"ticket_id": 1,
				})
				.build(),
		]
	}
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<Ticket>(), GenericCollection::new::<TicketMessage>()]
}
