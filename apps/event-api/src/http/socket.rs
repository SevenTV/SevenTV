use std::borrow::Cow;
use std::convert::Infallible;

use futures_util::{SinkExt, StreamExt};
use http_body::Frame;
use hyper::body::Bytes;
use hyper_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use hyper_tungstenite::tungstenite::protocol::CloseFrame;
use hyper_tungstenite::tungstenite::Message;
use hyper_tungstenite::{HyperWebsocket, HyperWebsocketStream};

/// A Socket is a wrapper around a websocket or SSE connection.
pub enum Socket {
	WebSocket(Box<WebSocket>),
	Sse(tokio::sync::mpsc::Sender<Result<Frame<Bytes>, Infallible>>),
}

/// Internally a websocket is a state machine, so we have to keep track of the
/// state.
pub enum WebSocket {
	Pending(HyperWebsocket),
	Ready(HyperWebsocketStream),
}

impl WebSocket {
	/// Wait for the websocket to be ready.
	pub async fn ready(&mut self) -> Result<(), hyper_tungstenite::tungstenite::Error> {
		match self {
			Self::Pending(ws) => {
				tracing::debug!("websocket pending");
				let ws = ws.await?;
				tracing::debug!("websocket ready");
				*self = Self::Ready(ws);
			}
			Self::Ready(_) => {}
		}

		Ok(())
	}

	/// Send a message over the websocket, this will wait for the websocket to
	/// be ready.
	pub async fn send(&mut self, data: impl SocketMessage) -> Result<(), hyper_tungstenite::tungstenite::Error> {
		// Wait for the websocket to be ready.
		self.ready().await?;

		match self {
			Self::Ready(ws) => {
				ws.send(data.into_ws()).await?;
			}
			_ => unreachable!("websocket not ready"),
		}

		Ok(())
	}

	/// Receive a message from the websocket, this will wait for the websocket
	/// to be ready.
	pub async fn recv(&mut self) -> Result<Message, hyper_tungstenite::tungstenite::Error> {
		// Wait for the websocket to be ready.
		self.ready().await?;

		match self {
			Self::Ready(ws) => Ok(ws
				.next()
				.await
				.ok_or(hyper_tungstenite::tungstenite::Error::ConnectionClosed)??),
			_ => unreachable!("websocket not ready"),
		}
	}

	/// Close the websocket, if the websocket is not ready, this will wait for
	/// it to be ready.
	pub async fn close(&mut self, close: Option<CloseFrame<'_>>) -> Result<(), hyper_tungstenite::tungstenite::Error> {
		// Wait for the websocket to be ready.
		self.ready().await?;

		match self {
			Self::Ready(ws) => {
				ws.close(close).await.ok();
			}
			_ => unreachable!("websocket not ready"),
		}

		// Not sure if this is needed, if we are the ones closing the websocket
		// however it doesn't hurt to flush it just in case.
		// See https://github.com/snapview/tungstenite-rs/issues/405
		self.flush().await.ok();

		Ok(())
	}

	/// Flush the websocket, if the websocket is not ready, this will wait for
	/// it to be ready.
	pub async fn flush(&mut self) -> Result<(), hyper_tungstenite::tungstenite::Error> {
		// Wait for the websocket to be ready.
		self.ready().await?;

		match self {
			Self::Ready(ws) => {
				ws.flush().await?;
			}
			_ => unreachable!("websocket not ready"),
		}

		Ok(())
	}
}

/// A trait for converting a message into a websocket or SSE message.
pub trait SocketMessage: Sized {
	fn into_sse(self) -> Frame<Bytes>;
	fn into_ws(self) -> Message;
}

impl SocketMessage for Message {
	fn into_sse(self) -> Frame<Bytes> {
		Frame::data(self.into_data().into())
	}

	fn into_ws(self) -> Message {
		self
	}
}

impl SocketMessage for Frame<Bytes> {
	fn into_sse(self) -> Frame<Bytes> {
		self
	}

	fn into_ws(self) -> Message {
		Message::binary(self.into_data().unwrap_or_default())
	}
}

impl SocketMessage for String {
	fn into_sse(self) -> Frame<Bytes> {
		Frame::data(self.into())
	}

	fn into_ws(self) -> Message {
		Message::text(self)
	}
}

impl SocketMessage for Bytes {
	fn into_sse(self) -> Frame<Bytes> {
		Frame::data(self)
	}

	fn into_ws(self) -> Message {
		Message::binary(self)
	}
}

impl SocketMessage for &[u8] {
	fn into_sse(self) -> Frame<Bytes> {
		Frame::data(Bytes::copy_from_slice(self))
	}

	fn into_ws(self) -> Message {
		Message::binary(self)
	}
}

impl SocketMessage for Vec<u8> {
	fn into_sse(self) -> Frame<Bytes> {
		Frame::data(self.into())
	}

	fn into_ws(self) -> Message {
		Message::binary(self)
	}
}

impl SocketMessage for &str {
	fn into_sse(self) -> Frame<Bytes> {
		Frame::data(Bytes::copy_from_slice(self.as_bytes()))
	}

	fn into_ws(self) -> Message {
		Message::text(self)
	}
}

#[derive(Debug, thiserror::Error)]
pub enum SocketError {
	#[error("websocket error: {0}")]
	WebSocket(#[from] hyper_tungstenite::tungstenite::Error),
	#[error("sse error, receiver dropped")]
	SseClosed,
}

impl Socket {
	/// Create a new socket from a websocket.
	pub fn websocket(ws: HyperWebsocket) -> Self {
		Self::WebSocket(Box::new(WebSocket::Pending(ws)))
	}

	/// Create a new socket from a SSE sender.
	pub fn sse(sender: tokio::sync::mpsc::Sender<Result<Frame<Bytes>, Infallible>>) -> Self {
		Self::Sse(sender)
	}

	/// Receive a message from the socket.
	pub async fn recv(&mut self) -> Result<Message, SocketError> {
		match self {
			Self::WebSocket(ws) => match ws.recv().await.map_err(SocketError::WebSocket) {
				Ok(Message::Close(frame)) => {
					// The tungstenite library will not send the echo back to the client
					// if we don't flush the socket. This is a bug in the library.
					// See https://github.com/snapview/tungstenite-rs/issues/405
					ws.flush().await?;
					Ok(Message::Close(frame))
				}
				r => r,
			},
			Self::Sse(socket) => {
				socket.closed().await;
				Ok(Message::Close(None))
			}
		}
	}

	/// Send a message over the socket.
	pub async fn send(&mut self, data: impl SocketMessage) -> Result<(), SocketError> {
		match self {
			Self::WebSocket(ws) => {
				ws.send(data).await?;
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
				ws.close(Some(CloseFrame {
					code,
					reason: Cow::Borrowed(reason),
				}))
				.await?;
			}
			Self::Sse(sse) => {
				sse.send(Ok(Frame::trailers(Default::default())))
					.await
					.map_err(|_| SocketError::SseClosed)?;
			}
		}

		Ok(())
	}
}
