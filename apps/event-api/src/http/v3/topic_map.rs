use std::pin::Pin;

use futures_util::Stream;

use crate::subscription::{Payload, SubscriberReceiver, TopicKey};

/// A Subscription is a dipatch receiver.
pub struct Subscription {
	auto: Option<u32>,
	subscriber: Pin<Box<SubscriberReceiver>>,
}

impl Subscription {
	/// Create a new subscription, with an optional auto value.
	pub fn new(auto: Option<u32>, subscriber: SubscriberReceiver) -> Self {
		Self {
			auto,
			subscriber: Box::pin(subscriber),
		}
	}

	/// Get the auto value.
	pub fn auto(&self) -> Option<u32> {
		self.auto
	}

	/// Set the auto value.
	pub fn set_auto(&mut self, auto: Option<u32>) {
		self.auto = auto;
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
pub struct TopicMap(Vec<(TopicKey, Subscription)>);

impl TopicMap {
	/// Insert a new subscription into the map.
	pub fn insert(&mut self, key: TopicKey, value: Subscription) {
		self.retain(|k, _| k != &key);
		self.0.push((key, value));
	}

	/// Retain all subscriptions that match the predicate.
	pub fn retain<F>(&mut self, mut f: F)
	where
		F: FnMut(&TopicKey, &mut Subscription) -> bool,
	{
		self.0.retain_mut(|(k, v)| f(k, v));
	}

	/// Get the length of the map.
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Shrinks the capacity of the map as much as possible.
	pub fn shrink_to_fit(&mut self) {
		self.0.shrink_to_fit();
	}

	/// Get a subscription by its key.
	pub fn get_mut(&mut self, key: &TopicKey) -> Option<&mut Subscription> {
		self.0.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
	}

	/// Remove a subscription by its key.
	pub fn remove(&mut self, key: &TopicKey) -> Option<Subscription> {
		let idx = self.0.iter().position(|(k, _)| k == key)?;
		Some(self.0.swap_remove(idx).1)
	}

	/// Poll the entries in the map.
	fn poll_entries(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Payload>> {
		// Start at the beginning of the vector.
		let mut idx = 0;
		loop {
			// If we reached the end of the vector, return pending.
			if idx >= self.0.len() {
				break std::task::Poll::Pending;
			}

			// Get the subscription at the current index.
			let (_, subscription) = &mut self.0[idx];
			// Poll the subscription.
			match subscription.subscriber.as_mut().poll_next(cx) {
				// If the subscription is ready, return the message.
				std::task::Poll::Ready(Some(msg)) => break std::task::Poll::Ready(Some(msg)),
				// If the subscription yielded none, remove it from the vector, because it is done.
				std::task::Poll::Ready(None) => {
					self.0.swap_remove(idx);
					continue;
				}
				// If the subscription is not ready, continue to the next subscription.
				std::task::Poll::Pending => {}
			}

			idx += 1;
		}
	}
}

// This does not need to be pinned because it can be polled without moving it.
impl Unpin for TopicMap {}

/// Allows iterating over the subscriptions in the map.
impl Stream for TopicMap {
	type Item = Payload;

	fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
		self.get_mut().poll_entries(cx)
	}
}
