use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use scuffle_utils::context::ContextExt;
use scuffle_utils::prelude::FutureTimeout;
use tokio::net::TcpSocket;

pub trait HealthProvider {
	fn ctx(&self) -> &scuffle_utils::context::Context;
	fn healthy(&self, path: &str) -> bool;
	fn bind(&self) -> std::net::SocketAddr;
}

impl<T: HealthProvider> HealthProvider for Arc<T> {
	fn ctx(&self) -> &scuffle_utils::context::Context {
		(**self).ctx()
	}

	fn healthy(&self, path: &str) -> bool {
		(**self).healthy(path)
	}

	fn bind(&self) -> std::net::SocketAddr {
		(**self).bind()
	}
}

pub async fn run(provider: impl HealthProvider) -> anyhow::Result<()> {
	let bind = provider.bind();
	tracing::info!("[health] listening on http://{}", bind);
	let socket = if bind.is_ipv6() {
		TcpSocket::new_v6()?
	} else {
		TcpSocket::new_v4()?
	};

	socket.set_reuseaddr(true).context("socket reuseaddr")?;
	socket.set_reuseport(true).context("socket reuseport")?;
	socket.bind(bind).context("socket bind")?;
	let listener = socket.listen(16)?;

	while let Ok(r) = listener.accept().context(provider.ctx()).await {
		handle_socket(&provider, r).await?;
	}

	Ok(())
}

async fn handle_socket(
	provider: &impl HealthProvider,
	r: std::io::Result<(tokio::net::TcpStream, std::net::SocketAddr)>,
) -> anyhow::Result<()> {
	let (socket, _) = r?;

	let service = service_fn(move |req| async move {
		Ok::<_, anyhow::Error>({
			let resp = hyper::Response::builder()
				.status(if provider.healthy(req.uri().path()) {
					StatusCode::OK
				} else {
					StatusCode::SERVICE_UNAVAILABLE
				})
				.body(Full::new(Bytes::new()))
				.unwrap();

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
