use std::collections::HashMap;
use std::num::NonZeroUsize;
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
use scuffle_foundations::telemetry::metrics::metrics;
use shared::database::badge::BadgeId;
use shared::database::emote_set::EmoteSetId;
use shared::database::paint::PaintId;
use shared::database::user::UserId;
use shared::database::Id;
use shared::event::InternalEventUserPresenceData;
use shared::event_api::payload::{Subscribe, SubscribeCondition};
use shared::event_api::types::{ChangeField, ChangeFieldType, ChangeMap, CloseCode, EventType, ObjectKind, Opcode};
use shared::event_api::{payload, Message, MessageData, MessagePayload};
use shared::old_types::cosmetic::{CosmeticBadgeModel, CosmeticKind, CosmeticModel, CosmeticPaintModel};
use shared::old_types::{EmotePartialModel, EmoteSetModel, Entitlement, EntitlementData, UserPartialModel};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use self::topic_map::TopicMap;
use super::socket::Socket;
use crate::global::{AtomicTicket, Global};
use crate::http::v3::error::ConnectionError;
use crate::http::v3::topic_map::Subscription;
use crate::subscription::{EventTopic, Payload};
use crate::utils::jitter;

pub mod error;
mod parser;
mod topic_map;

#[metrics]
mod v3 {
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::counter::Counter;
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::gauge::Gauge;
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::histogram::Histogram;
	use scuffle_foundations::telemetry::metrics::HistogramBuilder;
	use tokio::time::Instant;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
	#[serde(rename_all = "snake_case")]
	pub enum ConnectionKind {
		Websocket,
		EventStream,
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
	#[serde(rename_all = "snake_case")]
	pub enum CommandKind {
		Client,
		Server,
	}

	/// The current number of connections
	pub fn current_connections(kind: ConnectionKind) -> Gauge;

	pub struct CurrentConnectionDropGuard {
		kind: ConnectionKind,
	}

	impl Drop for CurrentConnectionDropGuard {
		fn drop(&mut self) {
			current_connections(self.kind).dec();
		}
	}

	impl CurrentConnectionDropGuard {
		pub fn new(kind: ConnectionKind) -> Self {
			current_connections(kind).inc();

			Self { kind }
		}
	}

	/// The number of client closes
	pub fn client_closes(code: &'static str, kind: ConnectionKind) -> Counter;

	/// The number of commands issued
	pub fn commands(kind: CommandKind, command: String) -> Counter;

	/// The number of seconds used on connections
	#[builder = HistogramBuilder::default()]
	pub fn connection_duration_seconds(kind: ConnectionKind) -> Histogram;

	pub struct ConnectionDurationDropGuard {
		kind: ConnectionKind,
		start: Instant,
	}

	impl ConnectionDurationDropGuard {
		pub fn new(kind: ConnectionKind) -> Self {
			Self {
				kind,
				start: Instant::now(),
			}
		}
	}

	impl Drop for ConnectionDurationDropGuard {
		fn drop(&mut self) {
			connection_duration_seconds(self.kind).observe(self.start.elapsed().as_secs_f64());
		}
	}
}

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
	let initial_subs = if let Some(query) = query.as_ref().and_then(|q| q.contains('@').then_some(q)) {
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
			.map_err(|_| (hyper::StatusCode::BAD_REQUEST, "failed to parse query"))?
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
	/// A map of subscriptions that this connection is subscribed to.
	topics: TopicMap,
	/// The initial subscriptions that this connection should subscribe to.
	initial_subs: Option<Vec<payload::Subscribe>>,
	/// Drop guard for the metrics
	_connection_duration_drop_guard: v3::ConnectionDurationDropGuard,
	/// Drop guard for the metrics
	_current_connection_drop_guard: v3::CurrentConnectionDropGuard,

	presence_lru: lru::LruCache<UserId, PresenceCacheValue>,
	personal_emote_set_lru: lru::LruCache<EmoteSetId, chrono::DateTime<chrono::Utc>>,
	badge_lru: lru::LruCache<BadgeId, chrono::DateTime<chrono::Utc>>,
	paint_lru: lru::LruCache<PaintId, chrono::DateTime<chrono::Utc>>,
}

#[derive(Default)]
struct PresenceCacheValue {
	personal_emote_sets: Vec<EmoteSetId>,
	active_badge: Option<BadgeId>,
	active_paint: Option<PaintId>,
}

/// The implementation of the socket.
impl Connection {
	pub fn new(
		socket: Socket,
		global: Arc<Global>,
		initial_subs: Option<Vec<payload::Subscribe>>,
		ticket: AtomicTicket,
	) -> Self {
		let connection_kind = match socket {
			Socket::WebSocket(_) => v3::ConnectionKind::Websocket,
			Socket::Sse(_) => v3::ConnectionKind::EventStream,
		};

		Self {
			socket,
			seq: 0,
			heartbeat_count: 0,
			id: Id::new(),
			// We jitter the TTL to prevent all connections from expiring at the same time, which
			// would cause a thundering herd.
			ttl: Box::pin(tokio::time::sleep(jitter(global.config().api.ttl))),
			topics: TopicMap::default(),
			// Same as above for the heartbeat interval.
			heartbeat_interval: tokio::time::interval(jitter(global.config().api.heartbeat_interval)),
			// And again for the subscription cleanup interval.
			initial_subs,
			_ticket: ticket,
			global,
			_connection_duration_drop_guard: v3::ConnectionDurationDropGuard::new(connection_kind),
			_current_connection_drop_guard: v3::CurrentConnectionDropGuard::new(connection_kind),
			presence_lru: lru::LruCache::new(NonZeroUsize::new(1024).unwrap()),
			badge_lru: lru::LruCache::new(NonZeroUsize::new(60).unwrap()),
			paint_lru: lru::LruCache::new(NonZeroUsize::new(250).unwrap()),
			personal_emote_set_lru: lru::LruCache::new(NonZeroUsize::new(1024).unwrap()),
		}
	}

	/// The entry point for the socket.
	async fn serve(mut self) {
		let ctx = scuffle_foundations::context::Context::global();

		// Send the hello message.
		match self
			.send_message(payload::Hello {
				heartbeat_interval: self.heartbeat_interval.period().as_millis() as u32,
				session_id: self.id,
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
						v3::client_closes(err.as_code(), v3::ConnectionKind::EventStream).inc();
					}
					Socket::WebSocket(_) => {
						v3::client_closes(err.as_code(), v3::ConnectionKind::Websocket).inc();
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
		v3::commands(v3::CommandKind::Server, data.opcode().to_string()).inc();
		self.socket.send(Message::new(data, self.seq - 1)).await?;
		Ok(())
	}

	/// Handle a subscription request.
	async fn handle_subscription(&mut self, subscribe: &payload::Subscribe) -> Result<(), ConnectionError> {
		if let Some(subscription_limit) = self.global.config().api.subscription_limit {
			if self.topics.len() >= subscription_limit {
				self.send_error("Too Many Active Subscriptions!", HashMap::new(), Some(CloseCode::RateLimit))
					.await?;
			}
		}

		let scope = match subscribe.condition.clone().try_into() {
			Ok(scope) => scope,
			Err(()) => {
				self.send_error(
					"Invalid Subscription Condition",
					HashMap::new(),
					Some(CloseCode::InvalidPayload),
				)
				.await?;
				return Ok(());
			}
		};

		let topic = EventTopic::new(subscribe.ty, scope);

		let topic_key = topic.as_key();

		if self.topics.contains_key(&topic_key) {
			self.send_error(
				"Already subscribed to this event",
				HashMap::new(),
				Some(CloseCode::AlreadySubscribed),
			)
			.await?;
		}

		self.topics.insert(
			topic_key,
			Subscription::new(self.global.subscription_manager().subscribe(topic).await?),
		);

		self.send_ack(
			Opcode::Subscribe,
			serde_json::json!({
				"id": self.seq,
				"type": subscribe.ty.as_str(),
				"condition": subscribe.condition,
			}),
		)
		.await?;

		Ok(())
	}

	/// Handle an unsubscribe request.
	async fn handle_unsubscribe(&mut self, unsubscribe: &payload::Unsubscribe) -> Result<(), ConnectionError> {
		if matches!(unsubscribe.condition, SubscribeCondition::Unknown) {
			let count = self.topics.len();
			self.topics.remove_all(unsubscribe.ty);
			if count == self.topics.len() {
				self.send_error("Not subscribed to this event", HashMap::new(), Some(CloseCode::NotSubscribed))
					.await?;
			}
		} else {
			let topic = EventTopic::new(
				unsubscribe.ty,
				match unsubscribe.condition.clone().try_into() {
					Ok(scope) => scope,
					Err(()) => {
						self.send_error(
							"Invalid Subscription Condition",
							HashMap::new(),
							Some(CloseCode::InvalidPayload),
						)
						.await?;
						return Ok(());
					}
				},
			)
			.as_key();

			if self.topics.remove(&topic).is_none() {
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
		v3::commands(v3::CommandKind::Client, msg.opcode.to_string()).inc();

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
				self.handle_subscription(&subscribe).await?;
			}
			MessageData::Unsubscribe(unsubscribe) => {
				self.handle_unsubscribe(&unsubscribe).await?;
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
		// If everything is good, send the dispatch to the client.
		self.send_message(&payload.data).await?;

		Ok(())
	}

	async fn handle_presence(&mut self, payload: &InternalEventUserPresenceData) -> Result<(), ConnectionError> {
		let mut dispatches = vec![];

		if let Some(badge) = payload.active_badge.as_ref() {
			if self.badge_lru.get(&badge.id).map(|t| t != &badge.updated_at).unwrap_or(true) {
				let object = CosmeticModel {
					id: badge.id,
					data: CosmeticBadgeModel::from_db(badge.clone(), &self.global.config().api.cdn_origin),
					kind: CosmeticKind::Badge,
				};
				let object = serde_json::to_value(object).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize badge");
					ConnectionError::ClosedByServer(CloseCode::ServerError)
				})?;

				dispatches.push(payload::Dispatch {
					ty: EventType::CreateCosmetic,
					body: ChangeMap {
						id: badge.id.cast(),
						kind: ObjectKind::Cosmetic,
						object,
						..Default::default()
					},
				});
			}

			self.badge_lru.put(badge.id, badge.updated_at);
		}

		if let Some(paint) = payload.active_paint.as_ref() {
			if self.paint_lru.get(&paint.id).map(|t| t != &paint.updated_at).unwrap_or(true) {
				let object = CosmeticModel {
					id: paint.id,
					data: CosmeticPaintModel::from_db(paint.clone(), &self.global.config().api.cdn_origin),
					kind: CosmeticKind::Paint,
				};
				let object = serde_json::to_value(object).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize badge");
					ConnectionError::ClosedByServer(CloseCode::ServerError)
				})?;

				dispatches.push(payload::Dispatch {
					ty: EventType::CreateCosmetic,
					body: ChangeMap {
						id: paint.id.cast(),
						kind: ObjectKind::Cosmetic,
						object,
						..Default::default()
					},
				});
			}

			self.paint_lru.put(paint.id, paint.updated_at);
		}

		let partial_user = UserPartialModel::from_db(payload.user.clone(), None, None, &self.global.config().api.cdn_origin);

		for emote_set in &payload.personal_emote_sets {
			if self
				.personal_emote_set_lru
				.get(&emote_set.emote_set.id)
				.map(|t| t != &emote_set.emote_set.updated_at)
				.unwrap_or(true)
			{
				let object =
					EmoteSetModel::from_db(emote_set.emote_set.clone(), std::iter::empty(), Some(partial_user.clone()));
				let object = serde_json::to_value(object).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize emote set");
					ConnectionError::ClosedByServer(CloseCode::ServerError)
				})?;

				dispatches.push(payload::Dispatch {
					ty: EventType::CreateEmoteSet,
					body: ChangeMap {
						id: emote_set.emote_set.id.cast(),
						kind: ObjectKind::EmoteSet,
						object,
						..Default::default()
					},
				});

				let pushed = emote_set
					.emotes
					.iter()
					.enumerate()
					.map(|(i, emote)| {
						let value = EmotePartialModel::from_db(
							emote.clone(),
							Some(UserPartialModel::deleted_user()),
							&self.global.config().api.cdn_origin,
						);
						let value = serde_json::to_value(value).map_err(|e| {
							tracing::error!(error = %e, "failed to serialize emote");
							ConnectionError::ClosedByServer(CloseCode::ServerError)
						})?;

						Ok(ChangeField {
							key: "emotes".to_string(),
							index: Some(i),
							ty: ChangeFieldType::Object,
							value,
							..Default::default()
						})
					})
					.collect::<Result<Vec<_>, ConnectionError>>()?;

				dispatches.push(payload::Dispatch {
					ty: EventType::UpdateEmoteSet,
					body: ChangeMap {
						id: emote_set.emote_set.id.cast(),
						kind: ObjectKind::EmoteSet,
						object: serde_json::Value::Null,
						pushed,
						..Default::default()
					},
				});
			}

			self.personal_emote_set_lru
				.put(emote_set.emote_set.id, emote_set.emote_set.updated_at);
		}

		let user_state = self.presence_lru.get_or_insert_mut_ref(&payload.user.id, Default::default);

		if user_state.active_badge != payload.active_badge.as_ref().map(|b| b.id) {
			if let Some(active_badge) = user_state.active_badge {
				let object = Entitlement {
					id: Id::<()>::nil(),
					data: EntitlementData::Badge { ref_id: active_badge },
					user: partial_user.clone(),
				};
				let value = serde_json::to_value(&object).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize entitlement");
					ConnectionError::ClosedByServer(CloseCode::ServerError)
				})?;

				dispatches.push(payload::Dispatch {
					ty: EventType::DeleteEntitlement,
					body: ChangeMap {
						id: object.id,
						kind: ObjectKind::Entitlement,
						object: value,
						..Default::default()
					},
				});
			}

			if let Some(badge) = payload.active_badge.as_ref() {
				let object = Entitlement {
					id: Id::<()>::nil(),
					data: EntitlementData::Badge { ref_id: badge.id },
					user: partial_user.clone(),
				};
				let value = serde_json::to_value(&object).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize entitlement");
					ConnectionError::ClosedByServer(CloseCode::ServerError)
				})?;

				dispatches.push(payload::Dispatch {
					ty: EventType::CreateEntitlement,
					body: ChangeMap {
						id: object.id.cast(),
						kind: ObjectKind::Cosmetic,
						object: value,
						..Default::default()
					},
				});
			}

			user_state.active_badge = payload.active_badge.as_ref().map(|b| b.id);
		}

		if user_state.active_paint != payload.active_paint.as_ref().map(|b| b.id) {
			if let Some(active_paint) = user_state.active_paint {
				let object = Entitlement {
					id: Id::<()>::nil(),
					data: EntitlementData::Paint { ref_id: active_paint },
					user: partial_user.clone(),
				};
				let value = serde_json::to_value(&object).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize entitlement");
					ConnectionError::ClosedByServer(CloseCode::ServerError)
				})?;

				dispatches.push(payload::Dispatch {
					ty: EventType::DeleteEntitlement,
					body: ChangeMap {
						id: object.id,
						kind: ObjectKind::Entitlement,
						object: value,
						..Default::default()
					},
				});
			}

			if let Some(paint) = payload.active_paint.as_ref() {
				let object = Entitlement {
					id: Id::<()>::nil(),
					data: EntitlementData::Paint { ref_id: paint.id },
					user: partial_user.clone(),
				};
				let value = serde_json::to_value(&object).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize entitlement");
					ConnectionError::ClosedByServer(CloseCode::ServerError)
				})?;

				dispatches.push(payload::Dispatch {
					ty: EventType::CreateEntitlement,
					body: ChangeMap {
						id: object.id.cast(),
						kind: ObjectKind::Cosmetic,
						object: value,
						..Default::default()
					},
				});
			}

			user_state.active_paint = payload.active_paint.as_ref().map(|p| p.id);
		}

		// Added
		for sen in &payload.personal_emote_sets {
			if !user_state.personal_emote_sets.contains(&sen.emote_set.id) {
				let object = Entitlement {
					id: Id::<()>::nil(),
					data: EntitlementData::EmoteSet {
						ref_id: sen.emote_set.id,
					},
					user: partial_user.clone(),
				};
				let value = serde_json::to_value(&object).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize entitlement");
					ConnectionError::ClosedByServer(CloseCode::ServerError)
				})?;

				dispatches.push(payload::Dispatch {
					ty: EventType::CreateEntitlement,
					body: ChangeMap {
						id: object.id.cast(),
						kind: ObjectKind::Entitlement,
						object: value,
						..Default::default()
					},
				});
			}
		}

		// Removed
		for sen in &user_state.personal_emote_sets {
			if !payload.personal_emote_sets.iter().any(|e| *sen == e.emote_set.id) {
				let object = Entitlement {
					id: Id::<()>::nil(),
					data: EntitlementData::EmoteSet { ref_id: *sen },
					user: partial_user.clone(),
				};
				let value = serde_json::to_value(&object).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize entitlement");
					ConnectionError::ClosedByServer(CloseCode::ServerError)
				})?;

				dispatches.push(payload::Dispatch {
					ty: EventType::DeleteEntitlement,
					body: ChangeMap {
						id: object.id,
						kind: ObjectKind::Entitlement,
						object: value,
						..Default::default()
					},
				});
			}
		}

		for dispatch in dispatches {
			self.send_message(&dispatch).await?;
		}

		Ok(())
	}

	/// The main driver for the socket.
	async fn cycle(&mut self) -> Result<(), ConnectionError> {
		// On the first cycle, we subscribe to the initial subscriptions.
		if let Some(initial_subs) = self.initial_subs.take() {
			for sub in &initial_subs {
				self.handle_subscription(sub).await?;
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
				match payload {
					Payload::Dispatch(payload) => self.handle_dispatch(payload.as_ref()).await,
					Payload::Presence(payload) => self.handle_presence(payload.as_ref()).await,
				}
			}
			_ = self.heartbeat_interval.tick() => {
				tracing::debug!("sending heartbeat");

				self.send_message(payload::Heartbeat {
					count: self.heartbeat_count,
				}).await?;

				self.heartbeat_count += 1;

				Ok(())
			},
			_ = &mut self.ttl => {
				tracing::debug!("ttl expired");
				Err(ConnectionError::TtlExpired)
			},
		}
	}
}
