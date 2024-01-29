use crate::http::socket::SocketError;
use crate::message::types::CloseCode;

#[derive(Debug, thiserror::Error)]
pub enum SocketV3Error {
	#[error("socket error: {0}")]
	Socket(#[from] SocketError),
	#[error("server is restarting")]
	GlobalClosed,
	#[error("max connection lifetime expired")]
	TtlExpired,
	#[error("client closed")]
	ClientClosed,
	#[error("invalid payload: {0}")]
	InvalidPayload(#[from] serde_json::Error),
	#[error("subscription error: {0}")]
	Subscription(#[from] crate::subscription::SubscriptionError),
	#[error("closed by server: {0}")]
	ClosedByServer(CloseCode),
	#[error("bridge error: {0}")]
	Bridge(#[from] reqwest::Error),
}

impl SocketV3Error {
	pub const fn as_str(&self) -> &'static str {
		match self {
			Self::Socket(_) => "CLIENT_CLOSED_ABNORMAL",
			Self::ClientClosed => "CLIENT_CLOSED_CLEAN",
			Self::Subscription(_) => "SUBSCRIPTION_ERROR",
			Self::Bridge(_) => "BRIDGE_ERROR",
			Self::InvalidPayload(_) => CloseCode::InvalidPayload.as_code_str(),
			Self::TtlExpired => CloseCode::Reconnect.as_code_str(),
			Self::GlobalClosed => CloseCode::Restart.as_code_str(),
			Self::ClosedByServer(code) => code.as_code_str(),
		}
	}

	pub const fn as_close_code(&self) -> Option<CloseCode> {
		match self {
			Self::Socket(_) => None,
			Self::ClientClosed => None,
			Self::Subscription(_) => Some(CloseCode::ServerError),
			Self::Bridge(_) => Some(CloseCode::ServerError),
			Self::InvalidPayload(_) => Some(CloseCode::InvalidPayload),
			Self::TtlExpired => Some(CloseCode::Reconnect),
			Self::GlobalClosed => Some(CloseCode::Restart),
			Self::ClosedByServer(code) => Some(*code),
		}
	}
}
