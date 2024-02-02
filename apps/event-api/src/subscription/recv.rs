use futures_util::Stream;
use tokio::sync::{broadcast, mpsc};

use super::{Event, EventTopic, Payload, TopicKey};

#[pin_project::pin_project]
/// Subscribe to a topic given its EventTopic.
/// This implements Stream, so you can use it like a normal stream
/// `sub.next().await`.
pub struct SubscriberReceiver {
	#[pin]
	rx: tokio_stream::wrappers::BroadcastStream<Payload>,
	_guard: SubscriberReceiverGuard,
}

/// Internal guard to unsubscribe from a topic when the SubscriberReceiver is
/// dropped.
struct SubscriberReceiverGuard {
	topic: TopicKey,
	events_tx: mpsc::UnboundedSender<Event>,
}

impl Stream for SubscriberReceiver {
	type Item = Payload;

	fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
		let mut this = self.project();
		loop {
			break match this.rx.as_mut().poll_next(cx) {
				std::task::Poll::Ready(Some(Ok(msg))) => std::task::Poll::Ready(Some(msg)),
				std::task::Poll::Ready(Some(Err(_))) => continue,
				std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
				std::task::Poll::Pending => std::task::Poll::Pending,
			};
		}
	}
}

impl SubscriberReceiver {
	pub(super) fn new(topic: EventTopic, rx: broadcast::Receiver<Payload>, unsub: mpsc::UnboundedSender<Event>) -> Self {
		Self {
			rx: tokio_stream::wrappers::BroadcastStream::new(rx),
			_guard: SubscriberReceiverGuard {
				topic: topic.as_key(),
				events_tx: unsub,
			},
		}
	}
}

impl Drop for SubscriberReceiverGuard {
	fn drop(&mut self) {
		self.events_tx.send(Event::Unsubscribe { topic: self.topic }).ok();
	}
}
