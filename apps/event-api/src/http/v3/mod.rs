use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Weak};

use http_body_util::{Full, StreamBody};
use hyper::body::Incoming;
use hyper_tungstenite::tungstenite::protocol::WebSocketConfig;
use scuffle_utils::http::ext::{OptionExt, ResultExt};
use scuffle_utils::http::router::ext::RequestExt;
use scuffle_utils::http::RouteError;
use tokio::time::Instant;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use ulid::Ulid;

use self::dedupe::Dedupe;
use self::parser::parse_query;
use self::topic_map::TopicMap;
use super::error::EventError;
use super::socket::Socket;
use super::Body;
use crate::global::{AtomicTicket, Global};
use crate::http::v3::error::SocketV3Error;
use crate::http::v3::topic_map::Subscription;
use crate::message::types::{CloseCode, Opcode};
use crate::message::{self, MessagePayload};
use crate::object_id::ObjectId;
use crate::subscription::EventTopic;
use crate::utils::{jitter, ContextExt};

mod dedupe;
pub mod error;
mod parser;
mod topic_map;

const MAX_CONDITIONS: usize = 10;
const MAX_CONDITION_KEY_LEN: usize = 64;
const MAX_CONDITION_VALUE_LEN: usize = 128;

/// Handle a request to the v3 API.
pub async fn handle(mut req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<EventError>> {
	let global = req
		.data::<Weak<Global>>()
		.expect("global missing")
		.upgrade()
		.map_err_route((hyper::StatusCode::INTERNAL_SERVER_ERROR, "shutdown"))?;

	// Get a connection ticket, this is used to track the number of active
	// connections.
	let (ticket, active) = global.inc_active_connections();
	if let Some(limit) = global.config().api.connection_limit {
		// if we exceed the connection limit, we return a 503.
		if active >= limit {
			tracing::debug!("connection limit reached: {} >= {limit}", active);
			return Ok(hyper::Response::builder()
				.status(hyper::StatusCode::SERVICE_UNAVAILABLE)
				.body(Body::Left(Full::default()))
				.expect("failed to build response"));
		}
	}

	// Parse the initial subscriptions from the path.
	let initial_subs = if req.uri().path().contains('@') {
		Some(parser::parse_path_subscriptions(
			&urlencoding::decode(req.uri().path()).unwrap_or_default(),
		))
	} else {
		parse_query(&req).map_ignore_err_route((hyper::StatusCode::BAD_REQUEST, "failed to parse query"))?
	};

	// Handle the websocket upgrade.
	if hyper_tungstenite::is_upgrade_request(&req) {
		let (response, websocket) = hyper_tungstenite::upgrade(
			&mut req,
			Some(WebSocketConfig {
				max_frame_size: Some(1024 * 16),
				max_message_size: Some(1024 * 18),
				write_buffer_size: 1024 * 16,
				..Default::default()
			}),
		)
		.map_err_route((hyper::StatusCode::INTERNAL_SERVER_ERROR, "failed to upgrade websocket"))?;

		global.metrics().incr_current_websocket_connections();
		let socket = SocketV3::new(Socket::websocket(websocket), global, initial_subs, ticket);

		tokio::spawn(socket.serve());

		Ok(response.map(Body::Left))
	} else if req.method() == hyper::Method::GET {
		// Handle the SSE request.
		let (sender, response) = tokio::sync::mpsc::channel(1);

		let response = Body::Right(StreamBody::new(ReceiverStream::new(response)));

		global.metrics().incr_current_event_streams();
		let socket = SocketV3::new(Socket::sse(sender), global, initial_subs, ticket);

		tokio::spawn(socket.serve());

		Ok(hyper::Response::builder()
			.header("Content-Type", "text/event-stream")
			.header("Cache-Control", "no-cache")
			.header("X-Accel-Buffering", "no")
			.body(response)
			.expect("failed to build response"))
	} else {
		// Return a 405 for any other method.
		Err((hyper::StatusCode::METHOD_NOT_ALLOWED, "method not allowed").into())
	}
}

struct SocketV3 {
	/// The socket that this connection is using.
	socket: Socket,
	/// The global state.
	global: Arc<Global>,
	/// The sequence counter for this connection.
	seq: u64,
	/// A ticket that is used to track the number of active connections.
	_ticket: AtomicTicket,
	/// The number of heartbeats that have been sent.
	heartbeat_count: u64,
	/// The ID of this connection.
	id: ObjectId,
	/// The TTL for this connection, in rust a Pin<Box<_>> is used to track the
	/// state of the timer.
	ttl: Pin<Box<tokio::time::Sleep>>,
	/// The interval for sending heartbeats.
	heartbeat_interval: tokio::time::Interval,
	/// The interval for cleaning up subscriptions, we auto unsubscribe from
	/// subscriptions that have been marked as auto.
	subscription_cleanup_interval: tokio::time::Interval,
	/// A map of subscriptions that this connection is subscribed to.
	topics: TopicMap,
	/// The initial subscriptions that this connection should subscribe to.
	initial_subs: Option<Vec<message::payload::Subscribe>>,
	/// A deduplication cache for dispatch events.
	dedupe: Dedupe,
	/// The time that this connection was started.
	start: Instant,
}

/// When the socket is dropped, we need to update the metrics.
/// This will always run regardless of how the socket was dropped (even during a
/// panic!)
impl Drop for SocketV3 {
	fn drop(&mut self) {
		match &self.socket {
			Socket::WebSocket(_) => {
				self.global.metrics().decr_current_websocket_connections();
				self.global
					.metrics()
					.observe_connection_duration_seconds_websocket(self.start.elapsed().as_secs_f64());
			}
			Socket::Sse(_) => {
				self.global.metrics().decr_current_event_streams();
				self.global
					.metrics()
					.observe_connection_duration_seconds_event_stream(self.start.elapsed().as_secs_f64());
			}
		}
	}
}

/// The implementation of the socket.
impl SocketV3 {
	pub fn new(
		socket: Socket,
		global: Arc<Global>,
		initial_subs: Option<Vec<message::payload::Subscribe>>,
		ticket: AtomicTicket,
	) -> Self {
		Self {
			socket,
			seq: 0,
			heartbeat_count: 0,
			id: ObjectId::from_ulid(Ulid::new()),
			// We jitter the TTL to prevent all connections from expiring at the same time, which would cause a thundering
			// herd.
			ttl: Box::pin(tokio::time::sleep(jitter(global.config().api.ttl))),
			topics: TopicMap::default(),
			// Same as above for the heartbeat interval.
			heartbeat_interval: tokio::time::interval(jitter(global.config().api.heartbeat_interval)),
			// And again for the subscription cleanup interval.
			subscription_cleanup_interval: tokio::time::interval(jitter(global.config().api.subscription_cleanup_interval)),
			initial_subs,
			dedupe: Dedupe::new(),
			_ticket: ticket,
			global,
			start: Instant::now(),
		}
	}

	/// The entry point for the socket.
	async fn serve(mut self) {
		let ctx = self.global.ctx().clone();

		// Send the hello message.
		match self
			.send_message(message::payload::Hello {
				heartbeat_interval: self.heartbeat_interval.period().as_millis() as u32,
				session_id: self.id,
				subscription_limit: self.global.config().api.subscription_limit.map(|s| s as i32).unwrap_or(-1),
				actor: None,
				instance: Some(message::payload::HelloInstanceInfo {
					name: "event-api".to_string(),
					population: self.global.active_connections() as i32,
				}),
			})
			.context(&ctx)
			.await
		{
			Ok(Ok(_)) => {}
			Ok(Err(err)) => {
				tracing::error!("socket error: {:?}", err);
				return;
			}
			Err(_) => {
				return;
			}
		}

		// The main loop for the socket.
		while match self.cycle().context(&ctx).await {
			Ok(Ok(_)) => true,
			r => {
				let err = r.unwrap_or(Err(SocketV3Error::GlobalClosed)).unwrap_err();

				if let Some(code) = err.as_close_code() {
					match code {
						CloseCode::Reconnect => {
							self.send_message(message::payload::Reconnect {
								reason: code.to_string(),
							})
							.await
							.ok();
						}
						CloseCode::Restart => {
							self.send_message(message::payload::Reconnect {
								reason: code.to_string(),
							})
							.await
							.ok();
						}
						_ => {}
					}

					if matches!(code, CloseCode::ServerError) {
						tracing::error!("socket error: {:?}", err);
					} else {
						tracing::debug!("socket closed: {:?}", err);
					}

					if !matches!(err, SocketV3Error::ClosedByServer(_)) {
						self.send_close(code).await.ok();
					}
				} else {
					tracing::debug!("socket closed: {:?}", err);
				}

				match self.socket {
					Socket::Sse(_) => {
						self.global.metrics().observe_client_close_event_stream(err.as_str());
					}
					Socket::WebSocket(_) => {
						self.global.metrics().observe_client_close_websocket(err.as_str());
					}
				}
				false
			}
		} {
			tracing::debug!("socket cycle");
		}
	}

	/// Send a close message to the client, and then close the socket.
	async fn send_close(&mut self, code: CloseCode) -> Result<(), SocketV3Error> {
		self.send_message(message::payload::EndOfStream {
			code,
			message: code.as_str().to_owned(),
		})
		.await?;
		self.socket.close(code.into_websocket(), code.as_str()).await?;
		Err(SocketV3Error::ClosedByServer(code))
	}

	/// Send an error message to the client, and then close the socket if a
	/// close code is provided.
	async fn send_error(
		&mut self,
		message: impl ToString,
		fields: HashMap<String, serde_json::Value>,
		close_code: Option<CloseCode>,
	) -> Result<(), SocketV3Error> {
		self.send_message(message::payload::Error {
			message: message.to_string(),
			fields,
			message_locale: None,
		})
		.await?;

		if let Some(close_code) = close_code {
			self.send_close(close_code).await?;
		}

		Ok(())
	}

	/// Send an ack message to the client.
	async fn send_ack(&mut self, command: Opcode, data: serde_json::Value) -> Result<(), SocketV3Error> {
		self.send_message(message::payload::Ack {
			command: command.to_string(),
			data,
		})
		.await
	}

	/// Send a message to the client.
	async fn send_message(&mut self, data: impl MessagePayload + serde::Serialize) -> Result<(), SocketV3Error> {
		self.seq += 1;
		self.global.metrics().observe_server_command(data.opcode());
		self.socket.send(message::Message::new(data, self.seq - 1)).await?;
		Ok(())
	}

	/// Handle a subscription request.
	async fn handle_subscription(
		&mut self,
		auto: Option<u32>,
		subscribe: &message::payload::Subscribe,
	) -> Result<(), SocketV3Error> {
		if subscribe.condition.is_empty() {
			self.send_error(
				"Wildcard event target subscription requires authentication",
				HashMap::new(),
				Some(CloseCode::InsufficientPrivilege),
			)
			.await?;
		}

		if let Some(subscription_limit) = self.global.config().api.subscription_limit {
			if self.topics.len() >= subscription_limit {
				self.send_error("Too Many Active Subscriptions!", HashMap::new(), Some(CloseCode::RateLimit))
					.await?;
			}
		}

		if subscribe.condition.len() > MAX_CONDITIONS {
			self.send_error(
				"Subscription Condition Too Large",
				vec![
					("condition_keys".to_string(), serde_json::json!(subscribe.condition.len())),
					("condition_keys_most".to_string(), serde_json::json!(MAX_CONDITIONS)),
				]
				.into_iter()
				.collect(),
				Some(CloseCode::RateLimit),
			)
			.await?;
		}

		if let Some((key, value)) = subscribe
			.condition
			.iter()
			.find(|(k, v)| k.len() > MAX_CONDITION_KEY_LEN || v.len() > MAX_CONDITION_VALUE_LEN)
		{
			self.send_error(
				"Subscription Condition Too Large",
				vec![
					("key".to_string(), serde_json::json!(key)),
					("value".to_string(), serde_json::json!(value)),
					("key_length".to_string(), serde_json::json!(key.len())),
					("value_length".to_string(), serde_json::json!(value.len())),
					("key_length_most".to_string(), serde_json::json!(MAX_CONDITION_KEY_LEN)),
					("value_length_most".to_string(), serde_json::json!(MAX_CONDITION_VALUE_LEN)),
				]
				.into_iter()
				.collect(),
				Some(CloseCode::RateLimit),
			)
			.await?;
		}

		let topic = EventTopic::new(subscribe.ty, &subscribe.condition);
		let topic_key = topic.as_key();
		match self.topics.get_mut(&topic_key) {
			Some(o) => {
				if o.auto().is_some() {
					o.set_auto(auto);
				} else if auto.is_none() {
					self.send_error(
						"Already subscribed to this event",
						HashMap::new(),
						Some(CloseCode::AlreadySubscribed),
					)
					.await?;
				}
			}
			None => {
				self.topics.insert(
					topic_key,
					Subscription::new(auto, self.global.subscription_manager().subscribe(topic).await?),
				);
			}
		}

		tracing::debug!("subscribed to event: {topic}");

		if auto.is_none() {
			self.send_ack(
				Opcode::Subscribe,
				serde_json::json!({
					"id": self.seq,
					"type": subscribe.ty.as_str(),
					"condition": subscribe.condition,
				}),
			)
			.await?;
		}

		Ok(())
	}

	/// Handle an unsubscribe request.
	async fn handle_unsubscribe(
		&mut self,
		auto: bool,
		unsubscribe: &message::payload::Unsubscribe,
	) -> Result<(), SocketV3Error> {
		if unsubscribe.condition.is_empty() {
			let count = self.topics.len();
			self.topics.retain(|topic, _| topic.0 != unsubscribe.ty);
			if count == self.topics.len() {
				if auto {
					return Ok(());
				}

				self.send_error("Not subscribed to this event", HashMap::new(), Some(CloseCode::NotSubscribed))
					.await?;
			}
		} else {
			let topic = EventTopic::new(unsubscribe.ty, &unsubscribe.condition).as_key();
			if self.topics.remove(&topic).is_none() {
				if auto {
					return Ok(());
				}

				self.send_error("Not subscribed to this event", HashMap::new(), Some(CloseCode::NotSubscribed))
					.await?;
			}
		}

		self.send_ack(
			Opcode::Unsubscribe,
			serde_json::json!({
				"type": unsubscribe.ty.as_str(),
				"condition": unsubscribe.condition,
			}),
		)
		.await?;

		Ok(())
	}

	/// Handle a message from the client.
	async fn handle_message(&mut self, msg: message::Message) -> Result<(), SocketV3Error> {
		self.global.metrics().observe_client_command(msg.opcode);

		// We match on the opcode so that we can deserialize the data into the correct
		// type.
		let msg = match msg.opcode {
			Opcode::Resume => {
				let msg = serde_json::from_value::<message::payload::Resume>(msg.data)?;
				message::MessageData::Resume(msg)
			}
			Opcode::Subscribe => {
				let msg = serde_json::from_value::<message::payload::Subscribe>(msg.data)?;
				message::MessageData::Subscribe(msg)
			}
			Opcode::Unsubscribe => {
				let msg = serde_json::from_value::<message::payload::Unsubscribe>(msg.data)?;
				message::MessageData::Unsubscribe(msg)
			}
			Opcode::Bridge => {
				let msg = serde_json::from_value::<message::payload::Bridge>(msg.data)?;
				message::MessageData::Bridge(msg)
			}
			_ => {
				self.send_error("Invalid Opcode", HashMap::new(), Some(CloseCode::UnknownOperation))
					.await?;
				return Ok(());
			}
		};

		match msg {
			message::MessageData::Resume(_) => {
				// Subscription resume is not supported.
				self.send_ack(
					Opcode::Resume,
					serde_json::json!({
						"success": false,
						"dispatches_replayed": 0,
						"subscriptions_restored": 0,
					}),
				)
				.await?;
			}
			message::MessageData::Subscribe(subscribe) => {
				self.handle_subscription(None, &subscribe).await?;
			}
			message::MessageData::Unsubscribe(unsubscribe) => {
				self.handle_unsubscribe(false, &unsubscribe).await?;
			}
			message::MessageData::Bridge(bridge) => {
				// Subscription bridge is a way of interacting with the API through the socket.
				let res = self
					.global
					.http_client()
					.post(&self.global.config().api.bridge_url)
					.json(&bridge.body)
					.send()
					.await?
					.error_for_status()?;
				let body = res.json::<Vec<message::Message<message::payload::Dispatch>>>().await?;

				for msg in body {
					self.handle_dispatch(&msg).await?;
				}
			}
			_ => {
				self.send_error("Invalid Opcode", HashMap::new(), Some(CloseCode::UnknownOperation))
					.await?;
			}
		}

		Ok(())
	}

	async fn handle_dispatch(
		&mut self,
		payload: &message::Message<message::payload::Dispatch>,
	) -> Result<(), SocketV3Error> {
		// Check if the dispatch is a whisper to a connection.
		if let Some(whisper) = &payload.data.whisper {
			if &self.id.to_string() != whisper {
				return Ok(());
			}
		}

		// Check if the dispatch is a duplicate.
		if let Some(hash) = payload.data.hash {
			if !self.dedupe.insert(hash) {
				return Ok(());
			}
		}

		// Check if the dispatch adds or removes subscriptions or hashes.
		if let Some(effect) = &payload.data.effect {
			for add in &effect.add_subscriptions {
				self.handle_subscription(Some(payload.data.hash.unwrap_or(0)), add).await?;
			}

			for remove in &effect.remove_subscriptions {
				self.handle_unsubscribe(true, remove).await?;
			}

			for hash in &effect.remove_hashes {
				self.dedupe.remove(hash);
			}
		}

		// If everything is good, send the dispatch to the client.
		self.send_message(&payload.data).await?;

		Ok(())
	}

	/// The main driver for the socket.
	async fn cycle(&mut self) -> Result<(), SocketV3Error> {
		// On the first cycle, we subscribe to the initial subscriptions.
		if let Some(initial_subs) = self.initial_subs.take() {
			for sub in &initial_subs {
				self.handle_subscription(None, sub).await?;
			}
		}

		// We use tokio::select! to wait for the first of the following to complete:
		// - A message from the client.
		// - A dispatch from the subscription manager.
		// - A heartbeat tick.
		// - A subscription cleanup tick.
		// - The TTL timer expiring.
		tokio::select! {
			r = self.socket.recv() => {
				let msg = r?;

				let msg = match msg {
					hyper_tungstenite::tungstenite::Message::Close(frame) => {
						tracing::debug!("received close message");
						return Err(SocketV3Error::ClientClosed(frame.map(|f| f.code)));
					}
					hyper_tungstenite::tungstenite::Message::Ping(payload) => {
						tracing::debug!("received ping message");
						self.socket.send(hyper_tungstenite::tungstenite::Message::Pong(payload)).await?;
						return Ok(());
					}
					hyper_tungstenite::tungstenite::Message::Text(txt) => {
						tracing::debug!("received text message");
						txt.into()
					}
					hyper_tungstenite::tungstenite::Message::Binary(bin) => {
						tracing::debug!("received binary message");
						bin
					}
					_ => return Ok(()),
				};

				self.handle_message(serde_json::from_slice(&msg)?).await
			},
			Some(payload) = self.topics.next() => {
				self.handle_dispatch(payload.as_ref()).await
			}
			_ = self.heartbeat_interval.tick() => {
				tracing::debug!("sending heartbeat");

				self.send_message(message::payload::Heartbeat {
					count: self.heartbeat_count,
				}).await?;

				self.heartbeat_count += 1;

				Ok(())
			},
			_ = self.subscription_cleanup_interval.tick() => {
				// When we receive a subscription cleanup tick, we remove all subscriptions that have been marked as auto.
				self.topics.retain(|_, s| {
					if let Some(auto) = s.auto() {
						self.dedupe.remove(&auto);
						false
					} else {
						true
					}
				});

				// We then shrink the topics map to free up memory.
				self.topics.shrink_to_fit();

				Ok(())
			},
			_ = &mut self.ttl => {
				tracing::debug!("ttl expired");
				Err(SocketV3Error::TtlExpired)
			},
		}
	}
}
