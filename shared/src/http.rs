use std::convert::Infallible;
use std::error::Error as StdError;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use http_body_util::{Full, StreamBody};
use hyper::body::{Bytes, Incoming};
use hyper::header::{self, HeaderValue};
use hyper::rt::{Read, Write};
use hyper::service::{service_fn, Service};
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use scuffle_utils::context::{Context, ContextExt};
use scuffle_utils::http::router::error::RouterError;
use scuffle_utils::http::router::middleware::Middleware;
use scuffle_utils::http::router::Router;
use scuffle_utils::prelude::FutureTimeout;
use tokio::net::{TcpSocket, TcpStream};
use tokio_rustls::TlsAcceptor;
use tokio_stream::wrappers::ReceiverStream;

use crate::config::{Http, HttpCors};

pub type Body =
	http_body_util::Either<Full<Bytes>, StreamBody<ReceiverStream<Result<hyper::body::Frame<Bytes>, Infallible>>>>;

/// Add CORS headers to the response.
pub fn cors_middleware<B: Sync + Send + 'static, E: 'static>(options: &HttpCors) -> Middleware<B, E> {
	let allow_origins = fnv::FnvHashSet::from_iter(options.allow_origin.iter().map(|s| s.to_lowercase()));
	let allow_methods = options.allow_methods.join(", ").parse::<HeaderValue>().unwrap();
	let allow_headers = options.allow_headers.join(", ").parse::<HeaderValue>().unwrap();
	let expose_headers = options.expose_headers.join(", ").parse::<HeaderValue>().unwrap();
	let max_age = options.max_age_seconds.map(|s| s.to_string().parse::<HeaderValue>().unwrap());
	let timing_allow_origins = fnv::FnvHashSet::from_iter(options.timing_allow_origin.iter().map(|s| s.to_lowercase()));

	let inner = move |resp: &mut Response<B>, req: &Request<()>| {
		if allow_origins.is_empty() {
			return;
		}

		let origin = match req.headers().get(header::ORIGIN) {
			Some(origin) => origin.clone(),
			None => return,
		};

		let origin_str = origin.to_str().unwrap();

		if !allow_origins.contains("*") && !allow_origins.contains(origin_str) {
			return;
		}

		resp.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());

		if timing_allow_origins.contains("*") || timing_allow_origins.contains(origin_str) {
			resp.headers_mut().insert("Timing-Allow-Origin", origin.clone());
		}

		if !allow_methods.is_empty() {
			resp.headers_mut()
				.insert(header::ACCESS_CONTROL_ALLOW_METHODS, allow_methods.clone());
		}

		if !allow_headers.is_empty() {
			resp.headers_mut()
				.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, allow_headers.clone());
		}

		if !expose_headers.is_empty() {
			resp.headers_mut()
				.insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, expose_headers.clone());
		}

		if let Some(max_age) = max_age.clone() {
			resp.headers_mut().insert(header::ACCESS_CONTROL_MAX_AGE, max_age);
		}

		resp.headers_mut().insert(header::VARY, "Origin".parse().unwrap());
	};

	Middleware::post_with_req(move |mut resp, req| {
		inner(&mut resp, &req);
		async move { Ok(resp) }
	})
}

pub trait HttpBuilder<B: hyper::body::Body + Send + Sync + 'static>: Send + Sync + 'static {
	fn serve<I, S>(
		&self,
		io: I,
		service: S,
	) -> impl std::future::Future<Output = Result<(), Box<dyn StdError + Send + Sync>>> + Send + '_
	where
		I: Read + Write + Send + Unpin + 'static,
		S: Service<Request<Incoming>, Response = Response<B>> + Send + 'static,
		S::Future: Send + 'static,
		B::Data: Send + Sync,
		B::Error: Into<Box<dyn StdError + Send + Sync>>,
		S::Error: Into<Box<dyn StdError + Send + Sync>>;

	fn supprt_http1(&self) -> bool {
		false
	}

	fn supprt_http2(&self) -> bool {
		false
	}

	fn supprt_http3(&self) -> bool {
		false
	}
}

impl<B: hyper::body::Body + Send + Sync + 'static> HttpBuilder<B> for hyper::server::conn::http1::Builder {
	async fn serve<I, S>(&self, io: I, service: S) -> Result<(), Box<dyn StdError + Send + Sync>>
	where
		I: Read + Write + Send + Unpin + 'static,
		S: Service<Request<Incoming>, Response = Response<B>> + Send + 'static,
		S::Future: Send + 'static,
		B::Data: Send + Sync,
		B::Error: Into<Box<dyn StdError + Send + Sync>>,
		S::Error: Into<Box<dyn StdError + Send + Sync>>,
	{
		self.serve_connection(io, service).with_upgrades().await?;
		Ok(())
	}

	fn supprt_http1(&self) -> bool {
		true
	}
}

impl<B: hyper::body::Body + Send + Sync + 'static> HttpBuilder<B> for hyper::server::conn::http2::Builder<TokioExecutor> {
	async fn serve<I, S>(&self, io: I, service: S) -> Result<(), Box<dyn StdError + Send + Sync>>
	where
		I: Read + Write + Send + Unpin + 'static,
		S: Service<Request<Incoming>, Response = Response<B>> + Send + 'static,
		S::Future: Send + 'static,
		B::Data: Send + Sync,
		B::Error: Into<Box<dyn StdError + Send + Sync>>,
		S::Error: Into<Box<dyn StdError + Send + Sync>>,
	{
		self.serve_connection(io, service).await?;
		Ok(())
	}

	fn supprt_http2(&self) -> bool {
		true
	}
}

impl<B: hyper::body::Body + Send + Sync + 'static> HttpBuilder<B>
	for hyper_util::server::conn::auto::Builder<TokioExecutor>
{
	async fn serve<I, S>(&self, io: I, service: S) -> Result<(), Box<dyn StdError + Send + Sync>>
	where
		I: Read + Write + Send + Unpin + 'static,
		S: Service<Request<Incoming>, Response = Response<B>> + Send + 'static,
		S::Future: Send + 'static,
		B::Data: Send + Sync,
		B::Error: Into<Box<dyn StdError + Send + Sync>>,
		S::Error: Into<Box<dyn StdError + Send + Sync>>,
	{
		self.serve_connection_with_upgrades(io, service).await?;
		Ok(())
	}

	fn supprt_http1(&self) -> bool {
		true
	}

	fn supprt_http2(&self) -> bool {
		true
	}
}

pub enum HttpAnyBuilder {
	Http1(hyper::server::conn::http1::Builder),
	Http2(hyper::server::conn::http2::Builder<TokioExecutor>),
	Auto(hyper_util::server::conn::auto::Builder<TokioExecutor>),
}

impl HttpAnyBuilder {
	pub fn new(config: &Http) -> Option<Self> {
		if config.http1.enabled && config.http2.enabled {
			let mut builder = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new());

			builder
				.http1()
				.half_close(config.http1.half_close)
				.keep_alive(config.http1.keep_alive)
				.max_buf_size(config.http1.max_buf_size)
				.timer(TokioTimer::new())
				.writev(config.http1.writev);

			if let Some(timeout) = config.http1.header_read_timeout {
				builder.http1().header_read_timeout(timeout);
			}

			builder
				.http2()
				.max_concurrent_streams(config.http2.max_concurrent_streams)
				.max_frame_size(config.http2.max_frame_size)
				.max_header_list_size(config.http2.max_header_list_size)
				.max_send_buf_size(config.http2.max_send_buf_size)
				.initial_stream_window_size(config.http2.initial_stream_window_size)
				.initial_connection_window_size(config.http2.initial_connection_window_size)
				.adaptive_window(config.http2.adaptive_window)
				.keep_alive_interval(config.http2.keep_alive_interval)
				.keep_alive_timeout(config.http2.keep_alive_timeout)
				.timer(TokioTimer::new());

			Some(Self::Auto(builder))
		} else if config.http1.enabled {
			let mut builder = hyper::server::conn::http1::Builder::new();

			builder
				.half_close(config.http1.half_close)
				.keep_alive(config.http1.keep_alive)
				.max_buf_size(config.http1.max_buf_size)
				.timer(TokioTimer::new())
				.writev(config.http1.writev);

			Some(Self::Http1(builder))
		} else if config.http2.enabled {
			let mut builder = hyper::server::conn::http2::Builder::new(TokioExecutor::new());

			builder
				.max_concurrent_streams(config.http2.max_concurrent_streams)
				.max_frame_size(config.http2.max_frame_size)
				.max_header_list_size(config.http2.max_header_list_size)
				.max_send_buf_size(config.http2.max_send_buf_size)
				.initial_stream_window_size(config.http2.initial_stream_window_size)
				.initial_connection_window_size(config.http2.initial_connection_window_size)
				.adaptive_window(config.http2.adaptive_window)
				.keep_alive_interval(config.http2.keep_alive_interval)
				.keep_alive_timeout(config.http2.keep_alive_timeout)
				.timer(TokioTimer::new());

			Some(Self::Http2(builder))
		} else {
			None
		}
	}
}

impl<B: hyper::body::Body + Send + Sync + 'static> HttpBuilder<B> for HttpAnyBuilder {
	async fn serve<I, S>(&self, io: I, service: S) -> Result<(), Box<dyn StdError + Send + Sync>>
	where
		I: Read + Write + Send + Unpin + 'static,
		S: Service<Request<Incoming>, Response = Response<B>> + Send + 'static,
		S::Future: Send + 'static,
		B::Data: Send + Sync,
		B::Error: Into<Box<dyn StdError + Send + Sync>>,
		S::Error: Into<Box<dyn StdError + Send + Sync>>,
	{
		match self {
			Self::Http1(builder) => builder.serve(io, service).await,
			Self::Http2(builder) => builder.serve(io, service).await,
			Self::Auto(builder) => builder.serve(io, service).await,
		}
	}

	fn supprt_http1(&self) -> bool {
		match self {
			Self::Http1(_) => true,
			Self::Http2(_) => false,
			Self::Auto(_) => true,
		}
	}

	fn supprt_http2(&self) -> bool {
		match self {
			Self::Http1(_) => false,
			Self::Http2(_) => true,
			Self::Auto(_) => true,
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum HandleSocketError {
	#[error("socket accept")]
	SocketAccept(#[source] std::io::Error),
	#[error("tls handshake")]
	TlsHandshake(#[source] std::io::Error),
	#[error("tls handshake timeout")]
	TlsHandshakeTimeout,
	#[error("http error")]
	Http(#[source] Box<dyn std::error::Error + Send + Sync>),
}

/// Create a new socket future.
#[tracing::instrument(name = "handle_socket", level = "debug", skip(handler, http_builder, tls_acceptor, socket))]
pub async fn handle_socket<B, E>(
	handler: Arc<impl HttpHandler<Body = B, Error = E> + 'static>,
	http_builder: Arc<impl HttpBuilder<B> + 'static>,
	tls_acceptor: Option<Arc<TlsAcceptor>>,
	socket: TcpStream,
	addr: SocketAddr,
) -> Result<(), HandleSocketError>
where
	B: hyper::body::Body + Send + Sync + 'static,
	E: Into<Box<dyn StdError + Send + Sync>>,
	B::Data: Send + Sync,
	B::Error: Into<Box<dyn StdError + Send + Sync>>,
{
	let service = {
		service_fn(move |mut req| {
			req.extensions_mut().insert(addr);
			let handler = handler.clone();
			async move { handler.handle(req).await }
		})
	};

	tracing::trace!("accepted connection");

	// // if we are using tls we need to do a tls handshake first
	if let Some(tls_acceptor) = tls_acceptor {
		let socket = match tls_acceptor.accept(socket).timeout(Duration::from_secs(5)).await {
			Ok(Ok(socket)) => socket,
			Ok(Err(err)) => {
				return Err(HandleSocketError::TlsHandshake(err));
			}
			Err(_) => {
				return Err(HandleSocketError::TlsHandshakeTimeout);
			}
		};

		tracing::debug!("tls handshake complete");

		// We need to allow upgrades because we have to upgrade to a websocket.
		http_builder.serve(TokioIo::new(socket), service).await
	} else {
		// See above.
		http_builder.serve(TokioIo::new(socket), service).await
	}
	.map_err(HandleSocketError::Http)?;

	tracing::debug!("connection closed");

	Ok(())
}

pub trait HttpHandler: Send + Sync {
	type Body: hyper::body::Body + Send + Sync + 'static;
	type Error: std::fmt::Debug + Send + Sync + 'static;

	fn handle(
		&self,
		req: Request<Incoming>,
	) -> impl std::future::Future<Output = Result<Response<Self::Body>, Self::Error>> + Send + '_;
}

impl<F, B, E, Fut> HttpHandler for F
where
	F: Send + Sync + 'static,
	F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
	Fut: std::future::Future<Output = Result<Response<B>, E>> + Send + 'static,
	B: hyper::body::Body + Send + Sync + 'static,
	E: std::fmt::Debug + Send + Sync + 'static,
{
	type Body = B;
	type Error = E;

	fn handle(&self, req: Request<Incoming>) -> impl std::future::Future<Output = Result<Response<B>, E>> + Send + '_ {
		(self)(req)
	}
}

impl<B, E> HttpHandler for Router<Incoming, B, E>
where
	B: hyper::body::Body + Send + Sync + 'static,
	E: std::fmt::Debug + Send + Sync + 'static,
{
	type Body = B;
	type Error = RouterError<E>;

	fn handle(
		&self,
		req: Request<Incoming>,
	) -> impl std::future::Future<Output = Result<Response<Self::Body>, Self::Error>> + Send + '_ {
		Router::handle(self, req)
	}
}

pub async fn run<B, E>(
	ctx: &Context,
	config: &Http,
	handler: impl HttpHandler<Body = B, Error = E> + 'static,
	accept_connection_fn: impl Fn(SocketAddr) -> bool + Send + Sync,
) -> anyhow::Result<()>
where
	B: hyper::body::Body + Send + Sync + 'static,
	E: Into<Box<dyn StdError + Send + Sync>> + 'static,
	B::Data: Send + Sync,
	B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
	let socket = if config.bind.is_ipv6() {
		TcpSocket::new_v6()?
	} else {
		TcpSocket::new_v4()?
	};

	let tls_acceptor = if let Some(tls) = &config.tls {
		tracing::debug!("tls enabled");
		let cert = tokio::fs::read(&tls.cert).await.context("ssl cert")?;
		let key = tokio::fs::read(&tls.key).await.context("ssl private key")?;

		let key = rustls_pemfile::pkcs8_private_keys(&mut std::io::BufReader::new(std::io::Cursor::new(key)))
			.next()
			.ok_or_else(|| anyhow::anyhow!("ssl private key missing"))?
			.context("ssl private key parse")?;

		let certs = rustls_pemfile::certs(&mut std::io::BufReader::new(std::io::Cursor::new(cert)))
			.collect::<Result<Vec<_>, _>>()
			.context("ssl cert parse")?;

		let mut tls_config = rustls::ServerConfig::builder()
			.with_no_client_auth()
			.with_single_cert(certs, key.into())
			.context("ssl config")?;

		if tls.alpn_protocols.is_empty() {
			tls_config.alpn_protocols = Vec::new();
			if config.http2.enabled {
				tls_config.alpn_protocols.push(b"h2".to_vec());
			}
			if config.http1.enabled {
				tls_config.alpn_protocols.push(b"http/1.1".to_vec());
			}
		} else {
			tls_config.alpn_protocols = tls.alpn_protocols.iter().map(|s| s.as_bytes().to_vec()).collect();
		}

		Some(Arc::new(tokio_rustls::TlsAcceptor::from(Arc::new(tls_config))))
	} else {
		tracing::debug!("tls disabled");
		None
	};

	socket.set_reuseaddr(config.reuse_addr).context("socket reuseaddr")?;
	socket.set_reuseport(config.reuse_port).context("socket reuseport")?;
	socket.bind(config.bind).context("socket bind")?;
	let listener = socket.listen(config.listen_backlog)?;

	tracing::info!(
		"listening on http{}://{}",
		if config.tls.is_some() { "s" } else { "" },
		listener.local_addr().context("socket local addr")?
	);

	let handler = Arc::new(handler);
	let http_builder = Arc::new(HttpAnyBuilder::new(config).context("all http versions disabled")?);

	while let Ok(r) = listener.accept().context(ctx).await {
		let (socket, addr) = r.context("socket accept")?;

		if !accept_connection_fn(addr) {
			tracing::debug!("connection rejected");
			continue;
		}

		let fut = handle_socket(handler.clone(), http_builder.clone(), tls_acceptor.clone(), socket, addr);

		tokio::spawn(async move {
			let Err(err) = fut.await else {
				tracing::debug!("connection closed");
				return;
			};

			match err {
				HandleSocketError::Http(err) => {
					tracing::error!("http error: {err:?}");
				}
				HandleSocketError::SocketAccept(err) => {
					tracing::warn!("socket accept error: {err}");
				}
				HandleSocketError::TlsHandshake(err) => {
					tracing::warn!("tls handshake error: {err}");
				}
				HandleSocketError::TlsHandshakeTimeout => {
					tracing::debug!("tls handshake timeout");
				}
			}
		});
	}

	Ok(())
}
