use super::v3::error::SocketV3Error;

#[derive(Debug, thiserror::Error)]
pub enum EventError {
	#[error("upgrade error: {0}")]
	Upgrade(#[from] hyper_tungstenite::tungstenite::error::ProtocolError),

	#[error("socket v3 error: {0}")]
	SocketV3(#[from] SocketV3Error),
}
