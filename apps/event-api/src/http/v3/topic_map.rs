use std::collections::BTreeMap;
use std::pin::Pin;

use futures_util::Stream;
use shared::event_api::types::EventType;

use crate::subscription::{Payload, SubscriberReceiver, TopicKey};

/// A Subscription is a dipatch receiver.
pub struct Subscription(Pin<Box<SubscriberReceiver>>);

impl Subscription {
	/// Create a new subscription, with an optional auto value.
	pub fn new(subscriber: SubscriberReceiver) -> Self {
		Self(Box::pin(subscriber))
	}
}

#[derive(Default)]
/// A TopicMap is a map of topics to subscriptions.
/// Implements a stream of payloads, which polls all subscriptions.
/// The reason we use a vector here instead of a hashmap is because we iterate
/// over the subscriptions very frequently, and the hashmap has a lot of
/// overhead for iterations. It costs more on memory aswell. The vector does not
/// do as well as the hashmap for lookups, inserts or deletions, but we don't do
/// those operations as frequently as we iterate over the subscriptions.
pub struct TopicMap(BTreeMap<TopicKey, Subscription>);

impl TopicMap {
	/// Insert a new subscription into the map.
	pub fn insert(&mut self, key: TopicKey, value: Subscription) {
		self.0.insert(key, value);
	}

	/// Get the length of the map.
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Remove a subscription by its key.
	pub fn remove(&mut self, key: &TopicKey) -> Option<Subscription> {
		self.0.remove(key)
	}

	pub fn contains_key(&self, key: &TopicKey) -> bool {
		self.0.contains_key(key)
	}

	pub fn remove_all(&mut self, key: EventType) {
		self.0.retain(|k, _| k.0 != key);
	}

	/// Poll the entries in the map.
	fn poll_entries(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Payload>> {
		let mut removals = Vec::new();
		let resp = self
			.0
			.iter_mut()
			.find_map(|(key, subscription)| match subscription.0.as_mut().poll_next(cx) {
				std::task::Poll::Ready(Some(msg)) => Some(std::task::Poll::Ready(Some(msg))),
				std::task::Poll::Ready(None) => {
					removals.push(*key);
					None
				}
				std::task::Poll::Pending => None,
			});

		removals.into_iter().for_each(|k| {
			self.0.remove(&k);
		});

		resp.unwrap_or(std::task::Poll::Pending)
	}
}

// This does not need to be pinned because the underlying subscriptions are
// already pinned.
impl Unpin for TopicMap {}

/// Allows iterating over the subscriptions in the map.
impl Stream for TopicMap {
	type Item = Payload;

	fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
		self.get_mut().poll_entries(cx)
	}
}
