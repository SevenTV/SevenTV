use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{ws, RawQuery, Request, State, WebSocketUpgrade};
use axum::http::header::HeaderMap;
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response, Sse};
use axum::routing::any;
use axum::Router;
use parser::{parse_json_subscriptions, parse_query_uri};
use scuffle_foundations::context::ContextFutExt;
use shared::database::Id;
use shared::event_api::payload::Subscribe;
use shared::event_api::types::{CloseCode, Opcode};
use shared::event_api::{payload, Message, MessageData, MessagePayload};
use tokio::time::Instant;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use self::dedupe::Dedupe;
use self::topic_map::TopicMap;
use super::socket::Socket;
use crate::global::{AtomicTicket, Global};
use crate::http::v3::error::ConnectionError;
use crate::http::v3::topic_map::Subscription;
use crate::subscription::EventTopic;
use crate::utils::jitter;

mod dedupe;
pub mod error;
mod parser;
mod topic_map;

const MAX_CONDITIONS: usize = 10;
const MAX_CONDITION_KEY_LEN: usize = 64;
const MAX_CONDITION_VALUE_LEN: usize = 128;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/", any(handle)).route("/*any", any(handle))
}

async fn handle(
	State(global): State<Arc<Global>>,
	RawQuery(query): RawQuery,
	headers: HeaderMap,
	upgrade: Option<WebSocketUpgrade>,
	req: Request,
) -> Result<Response<axum::body::Body>, (hyper::StatusCode, &'static str)> {
	let (ticket, active) = global.inc_active_connections();
	if let Some(limit) = global.config().api.connection_limit {
		// if we exceed the connection limit, we return a 503.
		if active >= limit {
			tracing::debug!("connection limit reached: {} >= {limit}", active);
			return Err((hyper::StatusCode::SERVICE_UNAVAILABLE, "connection limit reached"));
		}
	}

	// Parse the initial subscriptions from the path.
	let initial_subs = if let Some(query) = query.as_ref().and_then(|q| q.contains('@').then(|| q)) {
		Some(parser::parse_path_subscriptions(
			&urlencoding::decode(query).unwrap_or_default(),
		))
	} else {
		headers
			.get("x-7tv-query")
			.and_then(|v| v.to_str().ok().map(|s| s.to_string()))
			.or_else(|| parse_query_uri(query))
			.map(|q| parse_json_subscriptions(&q))
			.transpose()
			.map_err(|e| (hyper::StatusCode::BAD_REQUEST, "failed to parse query"))?
	};

	if let Some(upgrade) = upgrade {
		Ok(handle_ws(State(global), initial_subs, ticket, upgrade).await)
	} else if req.method() == hyper::Method::GET {
		handle_sse(State(global), initial_subs, ticket).await
	} else {
		Err((hyper::StatusCode::METHOD_NOT_ALLOWED, "method not allowed"))
	}
}

async fn handle_ws(
	State(global): State<Arc<Global>>,
	initial_subs: Option<Vec<Subscribe>>,
	ticket: AtomicTicket,
	upgrade: WebSocketUpgrade,
) -> Response<axum::body::Body> {
	upgrade
		.max_frame_size(1024 * 16)
		.max_message_size(1024 * 18)
		.write_buffer_size(1024 * 16)
		.on_upgrade(|ws| async {
			// global.metrics().incr_current_websocket_connections();
			let socket = Connection::new(Socket::websocket(ws), global, initial_subs, ticket);

			tokio::spawn(socket.serve());
		})
}

async fn handle_sse(
	State(global): State<Arc<Global>>,
	initial_subs: Option<Vec<Subscribe>>,
	ticket: AtomicTicket,
) -> Result<Response<axum::body::Body>, (hyper::StatusCode, &'static str)> {
	// Handle the SSE request.
	let (sender, response) = tokio::sync::mpsc::channel(1);

	let response = Sse::new(ReceiverStream::new(response));

	// global.metrics().incr_current_event_streams();
	let socket = Connection::new(Socket::sse(sender), global, initial_subs, ticket);

	tokio::spawn(socket.serve());

	let mut res = response.into_response();

	res.headers_mut()
		.append("Cache-Control", HeaderValue::from_static("no-cache"));
	res.headers_mut().append("X-Accel-Buffering", HeaderValue::from_static("no"));

	Ok(res)
}

struct Connection {
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
	id: Id,
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
	initial_subs: Option<Vec<payload::Subscribe>>,
	/// A deduplication cache for dispatch events.
	dedupe: Dedupe,
	/// The time that this connection was started.
	start: Instant,
}

/// When the socket is dropped, we need to update the metrics.
/// This will always run regardless of how the socket was dropped (even during a
/// panic!)
impl Drop for Connection {
	fn drop(&mut self) {
		match &self.socket {
			Socket::WebSocket(_) => {
				// self.global.metrics().decr_current_websocket_connections();
				// self.global
				// 	.metrics()
				// 	.observe_connection_duration_seconds_websocket(self.start.
				// elapsed().as_secs_f64());
			}
			Socket::Sse(_) => {
				// self.global.metrics().decr_current_event_streams();
				// self.global
				// 	.metrics()
				// 	.observe_connection_duration_seconds_event_stream(self.
				// start. elapsed().as_secs_f64());
			}
		}
	}
}

/// The implementation of the socket.
impl Connection {
	pub fn new(
		socket: Socket,
		global: Arc<Global>,
		initial_subs: Option<Vec<payload::Subscribe>>,
		ticket: AtomicTicket,
	) -> Self {
		Self {
			socket,
			seq: 0,
			heartbeat_count: 0,
			id: Id::new(),
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
		let ctx = scuffle_foundations::context::Context::global();

		// Send the hello message.
		match self
			.send_message(payload::Hello {
				heartbeat_interval: self.heartbeat_interval.period().as_millis() as u32,
				session_id: self.id.into(),
				subscription_limit: self.global.config().api.subscription_limit.map(|s| s as i32).unwrap_or(-1),
				actor: None,
				instance: Some(payload::HelloInstanceInfo {
					name: "event-api".to_string(),
					population: self.global.active_connections() as i32,
				}),
			})
			.with_context(&ctx)
			.await
		{
			Some(Ok(_)) => {}
			Some(Err(err)) => {
				tracing::error!("socket error: {:?}", err);
				return;
			}
			None => {
				return;
			}
		}

		// The main loop for the socket.
		while match self.cycle().with_context(&ctx).await {
			Some(Ok(_)) => true,
			r => {
				let err = r.unwrap_or(Err(ConnectionError::GlobalClosed)).unwrap_err();

				if let Some(code) = err.as_close_code() {
					match code {
						CloseCode::Reconnect => {
							self.send_message(payload::Reconnect {
								reason: code.to_string(),
							})
							.await
							.ok();
						}
						CloseCode::Restart => {
							self.send_message(payload::Reconnect {
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

					if !matches!(err, ConnectionError::ClosedByServer(_)) {
						self.send_close(code).await.ok();
					}
				} else {
					tracing::debug!("socket closed: {:?}", err);
				}

				match self.socket {
					Socket::Sse(_) => {
						// self.global.metrics().
						// observe_client_close_event_stream(err.as_str());
					}
					Socket::WebSocket(_) => {
						// self.global.metrics().
						// observe_client_close_websocket(err.as_str());
					}
				}
				false
			}
		} {
			tracing::debug!("socket cycle");
		}
	}

	/// Send a close message to the client, and then close the socket.
	async fn send_close(&mut self, code: CloseCode) -> Result<(), ConnectionError> {
		self.send_message(payload::EndOfStream {
			code,
			message: code.as_str().to_owned(),
		})
		.await?;
		self.socket.close(code, code.as_str()).await?;
		Err(ConnectionError::ClosedByServer(code))
	}

	/// Send an error message to the client, and then close the socket if a
	/// close code is provided.
	async fn send_error(
		&mut self,
		message: impl ToString,
		fields: HashMap<String, serde_json::Value>,
		close_code: Option<CloseCode>,
	) -> Result<(), ConnectionError> {
		self.send_message(payload::Error {
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
	async fn send_ack(&mut self, command: Opcode, data: serde_json::Value) -> Result<(), ConnectionError> {
		self.send_message(payload::Ack {
			command: command.to_string(),
			data,
		})
		.await
	}

	/// Send a message to the client.
	async fn send_message(&mut self, data: impl MessagePayload + serde::Serialize) -> Result<(), ConnectionError> {
		self.seq += 1;
		// self.global.metrics().observe_server_command(data.opcode());
		self.socket.send(Message::new(data, self.seq - 1)).await?;
		Ok(())
	}

	/// Handle a subscription request.
	async fn handle_subscription(
		&mut self,
		auto: Option<u32>,
		subscribe: &payload::Subscribe,
	) -> Result<(), ConnectionError> {
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
	async fn handle_unsubscribe(&mut self, auto: bool, unsubscribe: &payload::Unsubscribe) -> Result<(), ConnectionError> {
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
	async fn handle_message(&mut self, msg: Message) -> Result<(), ConnectionError> {
		// self.global.metrics().observe_client_command(msg.opcode);

		// We match on the opcode so that we can deserialize the data into the correct
		// type.
		let msg = match msg.opcode {
			Opcode::Resume => {
				let msg = serde_json::from_value::<payload::Resume>(msg.data)?;
				MessageData::Resume(msg)
			}
			Opcode::Subscribe => {
				let msg = serde_json::from_value::<payload::Subscribe>(msg.data)?;
				MessageData::Subscribe(msg)
			}
			Opcode::Unsubscribe => {
				let msg = serde_json::from_value::<payload::Unsubscribe>(msg.data)?;
				MessageData::Unsubscribe(msg)
			}
			Opcode::Bridge => {
				let msg = serde_json::from_value::<payload::Bridge>(msg.data)?;
				MessageData::Bridge(msg)
			}
			_ => {
				self.send_error("Invalid Opcode", HashMap::new(), Some(CloseCode::UnknownOperation))
					.await?;
				return Ok(());
			}
		};

		match msg {
			MessageData::Resume(_) => {
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
			MessageData::Subscribe(subscribe) => {
				self.handle_subscription(None, &subscribe).await?;
			}
			MessageData::Unsubscribe(unsubscribe) => {
				self.handle_unsubscribe(false, &unsubscribe).await?;
			}
			MessageData::Bridge(bridge) => {
				// Subscription bridge is a way of interacting with the API through the socket.
				let res = self
					.global
					.http_client()
					.post(&self.global.config().api.bridge_url)
					.json(&bridge.body)
					.send()
					.await?
					.error_for_status()?;
				let body = res.json::<Vec<Message<payload::Dispatch>>>().await?;

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

	async fn handle_dispatch(&mut self, payload: &Message<payload::Dispatch>) -> Result<(), ConnectionError> {
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
	async fn cycle(&mut self) -> Result<(), ConnectionError> {
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
					ws::Message::Close(frame) => {
						tracing::debug!("received close message");
						return Err(ConnectionError::ClientClosed(frame.map(|f| f.code)));
					}
					ws::Message::Ping(payload) => {
						tracing::debug!("received ping message");
						let answer = ws::Message::Pong(payload);
						self.socket.send(answer).await?;
						return Ok(());
					}
					ws::Message::Text(txt) => {
						tracing::debug!("received text message");
						txt.into()
					}
					ws::Message::Binary(bin) => {
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

				self.send_message(payload::Heartbeat {
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
				Err(ConnectionError::TtlExpired)
			},
		}
	}
}
