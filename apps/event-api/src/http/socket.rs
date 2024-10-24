use std::convert::Infallible;

use axum::extract::ws;
use axum::extract::ws::{CloseCode as WsCloseCode, CloseFrame, Message as WsMessage, WebSocket};
use axum::response::sse::Event;
use shared::event_api::types::CloseCode;
use shared::event_api::{Message, MessagePayload};

/// A Socket is a wrapper around a websocket or SSE connection.
pub enum Socket {
	WebSocket(Box<WebSocket>),
	Sse(tokio::sync::mpsc::Sender<Result<Event, Infallible>>),
}

/// A trait for converting a message into a websocket or SSE message.
pub trait SocketMessage: Sized {
	fn into_sse(self) -> Event;
	fn into_ws(self) -> WsMessage;
}

impl SocketMessage for WsMessage {
	fn into_sse(self) -> Event {
		panic!("cannot convert WsMessage into SSE")
	}

	fn into_ws(self) -> WsMessage {
		self
	}
}

impl<T: MessagePayload + serde::Serialize> SocketMessage for Message<T> {
	fn into_sse(self) -> Event {
		let data = serde_json::to_string(&self.data).expect("failed to serialize message");

		// Create a new frame with the data.
		Event::default()
			.event(self.opcode.as_str().to_lowercase())
			.data(data)
			.id(self.sequence.to_string())
	}

	fn into_ws(self) -> WsMessage {
		// Create a new frame with the data.
		WsMessage::Text(serde_json::to_string(&self).expect("failed to serialize message"))
	}
}

#[derive(Debug, thiserror::Error)]
pub enum SocketError {
	#[error("websocket error: {0}")]
	WebSocket(#[from] axum::Error),
	#[error("sse error, receiver dropped")]
	SseClosed,
	#[error("ws error, receiver dropped")]
	WebsocketClosed,
}

impl Socket {
	/// Create a new socket from a websocket.
	pub fn websocket(ws: WebSocket) -> Self {
		Self::WebSocket(Box::new(ws))
	}

	/// Create a new socket from a SSE sender.
	pub fn sse(sender: tokio::sync::mpsc::Sender<Result<Event, Infallible>>) -> Self {
		Self::Sse(sender)
	}

	/// Receive a message from the socket.
	pub async fn recv(&mut self) -> Result<WsMessage, SocketError> {
		match self {
			Self::WebSocket(ws) => match ws.recv().await.ok_or(SocketError::WebsocketClosed)? {
				Ok(WsMessage::Close(frame)) => {
					// The tungstenite library will not send the echo back to the client
					// if we don't flush the socket. This is a bug in the library.
					// See https://github.com/snapview/tungstenite-rs/issues/405
					// ws.flush().await?;
					Ok(WsMessage::Close(frame))
				}
				r => r.map_err(SocketError::WebSocket),
			},
			Self::Sse(socket) => {
				socket.closed().await;
				Ok(WsMessage::Close(None))
			}
		}
	}

	/// Send a message over the socket.
	pub async fn send(&mut self, data: impl SocketMessage) -> Result<(), SocketError> {
		match self {
			Self::WebSocket(ws) => {
				ws.send(data.into_ws()).await?;
			}
			Self::Sse(sender) => {
				sender.send(Ok(data.into_sse())).await.map_err(|_| SocketError::SseClosed)?;
			}
		}

		Ok(())
	}

	/// Close the socket.
	pub async fn close(&mut self, code: CloseCode, reason: &str) -> Result<(), SocketError> {
		match self {
			Self::WebSocket(ws) => {
				ws.send(ws::Message::Close(Some(CloseFrame {
					code: WsCloseCode::from(code.into_websocket()),
					reason: reason.to_owned().into(),
				})))
				.await?;
			}
			Self::Sse(sse) => {
				sse.send(Ok(Event::default().event("close")))
					.await
					.map_err(|_| SocketError::SseClosed)?;
			}
		}

		Ok(())
	}
}
