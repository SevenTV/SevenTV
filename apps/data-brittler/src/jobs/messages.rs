use std::sync::Arc;

use fnv::FnvHashMap;
use futures::TryStreamExt;
use mongodb::{
	bson::oid::ObjectId,
	options::InsertManyOptions,
};
use shared::database::{
	Collection, Ticket, TicketData, TicketMember, TicketMemberId, TicketMemberKind, TicketPriority, TicketStatus,
};

use crate::{error, global::Global, types};

use super::{Job, ProcessOutcome};

pub struct MessagesJob {
	global: Arc<Global>,
	read: FnvHashMap<ObjectId, bool>,

	tickets: Vec<Ticket>,
	ticket_members: Vec<TicketMember>,
}

impl Job for MessagesJob {
	const NAME: &'static str = "transfer_messages";

	type T = types::Message;

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		let mut read = FnvHashMap::default();

		tracing::info!("loading messages_read collection");
		let mut cursor = global
			.source_db()
			.collection::<types::MessageRead>("messages_read")
			.find(None, None)
			.await?;
		while let Some(message) = cursor.try_next().await? {
			read.insert(message.message_id, message.read);
		}

		Ok(Self {
			global,
			read,
			tickets: vec![],
			ticket_members: vec![],
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("messages")
	}

	async fn process(&mut self, message: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let id = message.id.into();

		let (data, title) = match message.data {
			types::MessageData::EmoteRequest {
				target_id,
				wish: Some(types::EmoteWish::List),
			} => (
				TicketData::EmoteListingRequest {
					emote_id: target_id.into(),
				},
				"Public Listing Request".to_string(),
			),
			types::MessageData::EmoteRequest {
				target_id,
				wish: Some(types::EmoteWish::PersonalUse),
			} => (
				TicketData::EmotePersonalUseRequest {
					emote_id: target_id.into(),
				},
				"Personal Use Request".to_string(),
			),
			// inbox messages are not tickets
			_ => return outcome,
		};

		let status = match self.read.get(&message.id) {
			Some(true) => TicketStatus::Fixed,
			Some(false) => TicketStatus::Pending,
			None => TicketStatus::InProgress,
		};

		self.tickets.push(Ticket {
			id,
			status,
			priority: TicketPriority::Low,
			title: title.clone(),
			tags: vec![],
			data,
		});

		self.ticket_members.push(TicketMember {
			id: TicketMemberId::new(),
			ticket_id: id,
			user_id: message.author_id.into(),
			kind: TicketMemberKind::Op,
			notifications: true,
			last_read: None,
		});

		if self.tickets.len() > 50_000 {
			let insert_options = InsertManyOptions::builder().ordered(false).build();
			let tickets = Ticket::collection(self.global.target_db());

			let Ok(res) = tickets.insert_many(&self.tickets, insert_options.clone()).await else {
				outcome.errors.push(error::Error::InsertMany);
				return outcome;
			};

			outcome.inserted_rows += res.inserted_ids.len() as u64;
			if res.inserted_ids.len() != self.tickets.len() {
				outcome.errors.push(error::Error::InsertMany);
			}

			self.tickets.clear();
		}

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing messages job");

		let mut outcome = ProcessOutcome::default();

		let insert_options = InsertManyOptions::builder().ordered(false).build();
		let tickets = Ticket::collection(self.global.target_db());
		let ticket_members = TicketMember::collection(self.global.target_db());

		let res = tokio::join!(
			tickets.insert_many(&self.tickets, insert_options.clone()),
			ticket_members.insert_many(&self.ticket_members, insert_options.clone()),
		);
		let res = vec![res.0, res.1].into_iter().zip(vec![
			self.tickets.len(),
			self.ticket_members.len(),
		]);

		for (res, len) in res {
			match res {
				Ok(res) => {
					outcome.inserted_rows += res.inserted_ids.len() as u64;
					if res.inserted_ids.len() != len {
						outcome.errors.push(error::Error::InsertMany);
					}
				}
				Err(e) => outcome.errors.push(e.into()),
			}
		}

		outcome
	}
}
