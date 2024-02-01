use super::v3::error::ConnectionError;

#[derive(Debug, thiserror::Error)]
pub enum EventError {
	#[error("upgrade error: {0}")]
	Upgrade(#[from] hyper_tungstenite::tungstenite::error::ProtocolError),

	#[error("connection error: {0}")]
	Connection(#[from] ConnectionError),
}
