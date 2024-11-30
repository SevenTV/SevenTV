use std::sync::Arc;

use metrics::{ActionKind, ConnectionDropGuard, SocketKind};
use ratelimit::{RateLimitDropGuard, RateLimiter};
use scuffle_http::body::IncomingBody;

pub mod ip;
pub mod ratelimit;

#[derive(Clone)]
pub struct MonitorHandler<H> {
	socket_kind: SocketKind,
	handle: H,
	_guard: Arc<(ConnectionDropGuard, Option<RateLimitDropGuard>)>,
}

#[async_trait::async_trait]
impl<H: scuffle_http::svc::ConnectionHandle> scuffle_http::svc::ConnectionHandle for MonitorHandler<H> {
	type Body = H::Body;
	type BodyData = H::BodyData;
	type BodyError = H::BodyError;
	type Error = H::Error;

	async fn on_request(&self, req: ::http::Request<IncomingBody>) -> Result<::http::Response<Self::Body>, Self::Error> {
		metrics::actions(self.socket_kind, ActionKind::Request).incr();
		self.handle.on_request(req).await
	}

	fn on_close(&self) {
		metrics::actions(self.socket_kind, ActionKind::Close).incr();
		self.handle.on_close();
	}

	fn on_error(&self, err: scuffle_http::Error) {
		metrics::actions(self.socket_kind, ActionKind::Error).incr();
		self.handle.on_error(err);
	}

	fn on_ready(&self) {
		metrics::actions(self.socket_kind, ActionKind::Ready).incr();
		self.handle.on_ready();
	}
}

#[derive(Clone)]
pub struct MonitorAcceptor<A> {
	inner: A,
	socket_kind: SocketKind,
	_limiter: Arc<RateLimiter>,
}

impl<A> MonitorAcceptor<A> {
	pub fn new(inner: A, socket_kind: SocketKind, limiter: Arc<RateLimiter>) -> Self {
		Self {
			inner,
			socket_kind,
			_limiter: limiter,
		}
	}
}

impl<A: scuffle_http::svc::ConnectionAcceptor> scuffle_http::svc::ConnectionAcceptor for MonitorAcceptor<A> {
	type Handle = MonitorHandler<A::Handle>;

	fn accept(&self, conn: scuffle_http::svc::IncomingConnection) -> Option<Self::Handle> {
		self.inner.accept(conn).map(|handle| MonitorHandler {
			handle,
			socket_kind: self.socket_kind,
			_guard: Arc::new((ConnectionDropGuard::new(self.socket_kind), None)),
		})
	}
}

#[scuffle_metrics::metrics(rename = "http")]
pub mod metrics {
	use scuffle_metrics::{CounterU64, HistogramF64, MetricEnum, UpDownCounterI64};

	pub struct ConnectionDropGuard(SocketKind);

	impl Drop for ConnectionDropGuard {
		fn drop(&mut self) {
			connections(self.0).decr();
		}
	}

	impl ConnectionDropGuard {
		pub fn new(socket: SocketKind) -> Self {
			connections(socket).incr();
			Self(socket)
		}
	}

	pub fn connections(socket: SocketKind) -> UpDownCounterI64;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, MetricEnum)]
	pub enum ActionKind {
		Error,
		Ready,
		Request,
		Close,
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, MetricEnum)]
	pub enum SocketKind {
		Tcp,
		TlsTcp,
		Quic,
	}

	pub fn actions(socket: SocketKind, action: ActionKind) -> CounterU64;

	pub fn status_code(socket: SocketKind, status: String) -> CounterU64;

	#[builder = HistogramBuilder::default()]
	pub fn socket_request_count(socket: SocketKind) -> HistogramF64;

	#[builder = HistogramBuilder::default()]
	pub fn socket_duration(socket: SocketKind) -> HistogramF64;

	#[builder = HistogramBuilder::default()]
	pub fn request_duration(socket: SocketKind) -> HistogramF64;

	pub fn bytes_sent(socket: SocketKind) -> CounterU64;
}
