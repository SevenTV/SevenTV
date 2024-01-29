use async_nats::SubscribeError;
use tokio::sync::oneshot;

#[derive(thiserror::Error, Debug)]
pub enum SubscriptionError {
	#[error("failed to send event")]
	SendEvent,
	#[error("failed to receive event: {0}")]
	RecvEvent(#[from] oneshot::error::RecvError),
	#[error("failed to subscribe to topic: {0}")]
	Nats(#[from] SubscribeError),
}
