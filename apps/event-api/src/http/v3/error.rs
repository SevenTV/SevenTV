use shared::event_api::types::CloseCode;

use crate::http::socket::SocketError;

type WsCloseCode = axum::extract::ws::CloseCode;

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
	#[error("socket error: {0}")]
	Socket(#[from] SocketError),
	#[error("server is restarting")]
	GlobalClosed,
	#[error("max connection lifetime expired")]
	TtlExpired,
	#[error("client closed")]
	ClientClosed(Option<WsCloseCode>),
	#[error("invalid payload: {0}")]
	InvalidPayload(#[from] serde_json::Error),
	#[error("subscription error: {0}")]
	Subscription(#[from] crate::subscription::SubscriptionError),
	#[error("closed by server: {0}")]
	ClosedByServer(CloseCode),
	#[error("bridge error: {0}")]
	Bridge(#[from] reqwest::Error),
}

impl ConnectionError {
	pub const fn as_str(&self) -> &'static str {
		match self {
			Self::Socket(_) => "CLIENT_CLOSED_ABNORMAL",
			Self::ClientClosed(code) => match code {
				None | Some(1000) => "CLIENT_CLOSED_CLEAN",
				Some(1001) => "CLIENT_CLOSED_AWAY",
				Some(1002) => "CLIENT_CLOSED_PROTOCOL",
				Some(1003) => "CLIENT_CLOSED_UNSUPPORTED",
				Some(1005) => "CLIENT_CLOSED_STATUS",
				Some(1006) => "CLIENT_CLOSED_ABNORMAL",
				Some(1007) => "CLIENT_CLOSED_INVALID",
				Some(1008) => "CLIENT_CLOSED_POLICY",
				Some(1009) => "CLIENT_CLOSED_SIZE",
				Some(1010) => "CLIENT_CLOSED_EXTENSION",
				Some(1011) => "CLIENT_CLOSED_ERROR",
				Some(1012) => "CLIENT_CLOSED_RESTART",
				Some(1013) => "CLIENT_CLOSED_AGAIN",
				_ => "CLIENT_CLOSED_UNKNOWN",
			},
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
			Self::ClientClosed(_) => None,
			Self::Subscription(_) => Some(CloseCode::ServerError),
			Self::Bridge(_) => Some(CloseCode::ServerError),
			Self::InvalidPayload(_) => Some(CloseCode::InvalidPayload),
			Self::TtlExpired => Some(CloseCode::Reconnect),
			Self::GlobalClosed => Some(CloseCode::Restart),
			Self::ClosedByServer(code) => Some(*code),
		}
	}
}
