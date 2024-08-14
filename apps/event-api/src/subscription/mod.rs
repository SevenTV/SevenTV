use std::sync::Arc;

use futures_util::StreamExt;
use shared::event::InternalEventPayload;
use shared::event_api::types::EventType;
use shared::event_api::{payload, Message};
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};

use crate::global::Global;

/// The payload of a message.
/// The reason we use Arc is because we want to avoid cloning the payload (1000
/// ish bytes) for every subscriber. Arc is a reference counted pointer, so it
/// is cheap to clone.
pub type Payload = Arc<Message<payload::Dispatch>>;

mod error;
mod event_topic;
mod recv;

pub use error::SubscriptionError;
pub use event_topic::{EventTopic, TopicKey};
pub use recv::SubscriberReceiver;

#[derive(Debug)]
/// An internal event for the SubscriptionManager, used to subscribe and
/// unsubscribe to topics from different contexts.
enum Event {
	Subscribe {
		topic: TopicKey,
		tx: oneshot::Sender<broadcast::Receiver<Payload>>,
	},
	Unsubscribe {
		topic: TopicKey,
	},
}

pub struct SubscriptionManager {
	events_tx: mpsc::UnboundedSender<Event>,
	events_rx: Mutex<mpsc::UnboundedReceiver<Event>>,
}

impl Default for SubscriptionManager {
	fn default() -> Self {
		// Only one value is needed in the channel.
		// This is a way to get around we cannot await in a drop.
		let (events_tx, events_rx) = mpsc::unbounded_channel();

		Self {
			events_rx: Mutex::new(events_rx),
			events_tx,
		}
	}
}

impl SubscriptionManager {
	fn events_tx(&self) -> &Mutex<mpsc::UnboundedReceiver<Event>> {
		&self.events_rx
	}

	/// Subscribe to a topic given its EventTopic.
	pub async fn subscribe(&self, topic: EventTopic) -> Result<SubscriberReceiver, SubscriptionError> {
		let (tx, rx) = oneshot::channel();

		self.events_tx
			.send(Event::Subscribe {
				topic: topic.as_key(),
				tx,
			})
			.map_err(|_| SubscriptionError::SendEvent)?;

		let rx = rx.await?;

		Ok(SubscriberReceiver::new(topic, rx, self.events_tx.clone()))
	}
}

/// The subscription manager run loop.
/// This function will block until the global context is done or when the NATS
/// connection is closed. Calling this function multiple times will deadlock.
pub async fn run(global: Arc<Global>) -> Result<(), SubscriptionError> {
	let mut events_rx = global.subscription_manager().events_tx().lock().await;

	// We subscribe to all events.
	// The .> wildcard is used to subscribe to all events.
	let mut sub = global.nats().subscribe("api.v4.events").await?;

	// fnv::FnvHashMap is used because it is faster than the default HashMap for our
	// use case.
	let mut subscriptions = fnv::FnvHashMap::default();

	let ctx = scuffle_foundations::context::Context::global();

	let mut seq = 0;

	loop {
		tokio::select! {
			Some(event) = events_rx.recv() => {
				match event {
					Event::Subscribe { topic, tx } => {
						match subscriptions.entry(topic) {
							std::collections::hash_map::Entry::Vacant(entry) => {
								let (btx, brx) = broadcast::channel(16);

								if tx.send(brx).is_ok() {
									// global.metrics().incr_total_subscriptions();
									entry.insert(btx);
								}
							}
							std::collections::hash_map::Entry::Occupied(entry) => {
								if tx.send(entry.get().subscribe()).is_ok() {
									// global.metrics().incr_total_subscriptions();
								}
							},
						}

						// global.metrics().set_unique_subscriptions(subscriptions.len(), subscriptions.capacity());
					}
					Event::Unsubscribe { topic } => {
						// global.metrics().decr_total_subscriptions();
						match subscriptions.entry(topic) {
							std::collections::hash_map::Entry::Occupied(entry) => {
								if entry.get().receiver_count() == 0 {
									entry.remove();
								}
							},
							std::collections::hash_map::Entry::Vacant(_) => {}
						}

						// global.metrics().set_unique_subscriptions(subscriptions.len(), subscriptions.capacity());
					}
				}
			}
			message = sub.next() => {
				match message {
					Some(message) => {
						let payload: InternalEventPayload = match rmp_serde::from_slice(&message.payload) {
							Ok(payload) => payload,
							Err(err) => {
								tracing::warn!(err = ?err, "malformed message");
								break;
							}
						};

						match payload.into_old_messages(&global.config().api.cdn_origin, seq) {
							Ok(messages) => {
								for message in messages {
									// There is always only one condition map
									let Some(condition) = message.data.condition.first() else {
										tracing::warn!("missing condition");
										continue;
									};

									let topic = EventTopic::new(message.data.ty, condition);

									let mut keys = vec![topic.as_key()];
									match keys[0].0 {
										EventType::SystemAnnouncement => {
											keys.push(topic.copy_cond(EventType::AnySystem).as_key());
										},
										EventType::CreateEmote | EventType::UpdateEmote | EventType::DeleteEmote => {
											keys.push(topic.copy_cond(EventType::AnyEmote).as_key());
										},
										EventType::CreateEmoteSet | EventType::UpdateEmoteSet | EventType::DeleteEmoteSet => {
											keys.push(topic.copy_cond(EventType::AnyEmoteSet).as_key());
										},
										EventType::CreateUser | EventType::UpdateUser | EventType::DeleteUser => {
											keys.push(topic.copy_cond(EventType::AnyUser).as_key());
										},
										EventType::CreateEntitlement | EventType::UpdateEntitlement | EventType::DeleteEntitlement => {
											keys.push(topic.copy_cond(EventType::AnyEntitlement).as_key());
										},
										EventType::CreateCosmetic | EventType::UpdateCosmetic | EventType::DeleteCosmetic => {
											keys.push(topic.copy_cond(EventType::AnyCosmetic).as_key());
										},
										EventType::Whisper => {}
										EventType::AnySystem | EventType::AnyEmote | EventType::AnyEmoteSet | EventType::AnyUser | EventType::AnyEntitlement | EventType::AnyCosmetic => {}
									}

									let message = Arc::new(message);

									let mut missed = true;
									for key in keys {
										if let std::collections::hash_map::Entry::Occupied(subscription) = subscriptions.entry(key) {
											if subscription.get().send(Arc::clone(&message)).is_err() {
												subscription.remove();
											} else {
												missed = false;
											}
										}
									}

									if missed {
										// global.metrics().observe_nats_event_miss();
									} else {
										// global.metrics().observe_nats_event_hit();
									}
								}
							},
							Err(err) => {
								tracing::warn!(error = %err, "failed to parse message");
							},
						}

						seq += 1;
					},
					None => {
						tracing::warn!("subscription closed");
						break;
					}
				}
			}
			_ = ctx.done() => {
				break;
			}
		}
	}

	Ok(())
}
