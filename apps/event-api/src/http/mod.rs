use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use futures_util::Future;
use http_body_util::{Full, StreamBody};
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{header, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use scuffle_utils::http::router::middleware::Middleware;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::{error_handler, RouteError};
use scuffle_utils::prelude::FutureTimeout;
use tokio::net::{TcpSocket, TcpStream};
use tokio_rustls::TlsAcceptor;
use tokio_stream::wrappers::ReceiverStream;

use self::error::EventError;
use crate::global::Global;

mod error;
pub mod socket;
pub mod v3;

type Body = http_body_util::Either<Full<Bytes>, StreamBody<ReceiverStream<Result<hyper::body::Frame<Bytes>, Infallible>>>>;

/// Add CORS headers to the response.
pub fn cors_middleware(_: &Arc<Global>) -> Middleware<Body, RouteError<EventError>> {
	// TODO: make this configurable
	Middleware::post(|mut resp| async move {
		resp.headers_mut()
			.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
		resp.headers_mut()
			.insert(header::ACCESS_CONTROL_ALLOW_METHODS, "*".parse().unwrap());
		resp.headers_mut()
			.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, "*".parse().unwrap());
		resp.headers_mut()
			.insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, "Date".parse().unwrap());
		resp.headers_mut().insert("Timing-Allow-Origin", "*".parse().unwrap());
		resp.headers_mut().insert(
			header::ACCESS_CONTROL_MAX_AGE,
			Duration::from_secs(86400).as_secs().to_string().parse().unwrap(),
		);

		Ok(resp)
	})
}

pub fn routes(global: &Arc<Global>) -> Router<Incoming, Body, RouteError<EventError>> {
	let weak = Arc::downgrade(global);

	Router::builder()
		.data(weak)
		.error_handler(|req, err| async move { error_handler(req, err).await.map(Body::Left) })
		.middleware(cors_middleware(global))
		// Handle the v3 API, we have to use a wildcard because of the path format.
		.any("/v3*", v3::handle)
		// Not found handler.
		.not_found(|_| async move { Err((StatusCode::NOT_FOUND, "not found").into()) })
		.build()
}

pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let config = global.config();
	tracing::info!(
		"[api] listening on http{}://{}",
		if config.api.tls.is_some() { "s" } else { "" },
		config.api.bind
	);
	let socket = if config.api.bind.is_ipv6() {
		TcpSocket::new_v6()?
	} else {
		TcpSocket::new_v4()?
	};

	let tls_acceptor = if let Some(tls) = &config.api.tls {
		tracing::info!("tls enabled");
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

		tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

		Some(Arc::new(tokio_rustls::TlsAcceptor::from(Arc::new(tls_config))))
	} else {
		tracing::info!("tls disabled");
		None
	};

	let router = Arc::new(routes(&global));

	socket.set_reuseaddr(true).context("socket reuseaddr")?;
	socket.set_reuseport(true).context("socket reuseport")?;
	socket.bind(config.api.bind).context("socket bind")?;
	let listener = socket.listen(128)?;

	loop {
		tokio::select! {
			_ = global.ctx().done() => {
				return Ok(());
			},
			r = listener.accept() => {
				if let Some(fut) = handle_socket(&global, &router, &tls_acceptor, r)? {
					tokio::spawn(fut);
				}
			},
		}
	}
}

/// Create a new socket future.
fn handle_socket(
	global: &Arc<Global>,
	router: &Arc<Router<Incoming, Body, RouteError<EventError>>>,
	tls_acceptor: &Option<Arc<TlsAcceptor>>,
	socket: std::io::Result<(TcpStream, SocketAddr)>,
) -> anyhow::Result<Option<impl Future<Output = ()>>> {
	let (socket, addr) = socket.context("socket accept")?;

	if let Some(limit) = global.config().api.connection_limit {
		let active = global.active_connections();
		if active >= limit {
			tracing::debug!("connection limit reached");
			return Ok(None);
		}
	}

	let router = router.clone();
	let service = service_fn(move |mut req| {
		req.extensions_mut().insert(addr);
		let this = router.clone();
		async move { this.handle(req).await }
	});

	let tls_acceptor = tls_acceptor.clone();

	tracing::trace!("accepted connection from {addr}");

	Ok(Some(async move {
		let mut http = Builder::new(TokioExecutor::new());

		// TODO: make this configurable
		http.http1().half_close(true).max_buf_size(16 * 1024).writev(true);

		http.http2()
			.max_concurrent_streams(1024)
			.max_frame_size(16 * 1024)
			.max_send_buf_size(16 * 1024)
			.max_header_list_size(16 * 1024);

		// if we are using tls we need to do a tls handshake first
		let result = if let Some(tls_acceptor) = tls_acceptor {
			let socket = match tls_acceptor.accept(socket).timeout(Duration::from_secs(5)).await {
				Ok(Ok(socket)) => socket,
				Ok(Err(err)) => {
					tracing::debug!("tls handshake error: {:?}", err);
					return;
				}
				Err(_) => {
					tracing::debug!("tls handshake timeout");
					return;
				}
			};

			tracing::debug!("tls handshake complete");

			// We need to allow upgrades because we have to upgrade to a websocket.
			http.serve_connection_with_upgrades(TokioIo::new(socket), service).await
		} else {
			// See above.
			http.serve_connection_with_upgrades(TokioIo::new(socket), service).await
		};

		if let Err(err) = result {
			tracing::debug!("connection error: {:?}", err);
		}
	}))
}
