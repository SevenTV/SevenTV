use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_nats::connection::State;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use scuffle_utils::context::ContextExt;
use scuffle_utils::prelude::FutureTimeout;
use tokio::net::TcpSocket;

use crate::global::Global;

pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let config = global.config();
	tracing::info!("[health] listening on http://{}", config.health.bind);
	let socket = if config.health.bind.is_ipv6() {
		TcpSocket::new_v6()?
	} else {
		TcpSocket::new_v4()?
	};

	socket.set_reuseaddr(true).context("socket reuseaddr")?;
	socket.set_reuseport(true).context("socket reuseport")?;
	socket.bind(config.health.bind).context("socket bind")?;
	let listener = socket.listen(16)?;

	while let Ok(r) = listener.accept().context(global.ctx()).await {
		handle_socket(&global, r).await?;
	}

	Ok(())
}

async fn handle_socket(
	global: &Arc<Global>,
	r: std::io::Result<(tokio::net::TcpStream, std::net::SocketAddr)>,
) -> anyhow::Result<()> {
	let config = global.config();
	let (socket, _) = r?;

	// Check if we have capacity for another connection.
	let capacity = if let Some(limit) = config.api.connection_target.or(config.api.connection_limit) {
		global.active_connections() < limit
	} else {
		true
	};

	let health = global.nats().connection_state() == State::Connected;

	let service = service_fn(move |req| async move {
		Ok::<_, anyhow::Error>({
			let resp = hyper::Response::builder();

			let resp = match req.uri().path() {
				"/capacity" => resp
					.status(if !capacity || !health {
						StatusCode::SERVICE_UNAVAILABLE
					} else {
						StatusCode::OK
					})
					.body(Full::new(Bytes::new()))
					.unwrap(),
				_ => resp
					.status(if health {
						StatusCode::OK
					} else {
						StatusCode::SERVICE_UNAVAILABLE
					})
					.body(Full::new(Bytes::new()))
					.unwrap(),
			};

			let user_agent = req
				.headers()
				.get("user-agent")
				.and_then(|v| v.to_str().ok())
				.unwrap_or("unknown");

			tracing::debug!("health check from {user_agent}: {}", resp.status());

			resp
		})
	});

	http1::Builder::new()
		.serve_connection(TokioIo::new(socket), service)
		.timeout(Duration::from_secs(2))
		.await
		.ok();

	Ok(())
}
