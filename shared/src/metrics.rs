use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use prometheus_client::encoding::{EncodeLabelKey, EncodeLabelSet, EncodeLabelValue};
use scuffle_utils::context::ContextExt;
use scuffle_utils::prelude::FutureTimeout;
use tokio::net::TcpSocket;

pub const DEFAULT_HISTOGRAM_BUCKETS: &[f64] = &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// Labels<T> is a wrapper around a set of labels.
/// This is advantageous because it allows us to have a base set of labels that
/// are always present, and then extend them with additional labels. Without
/// copying the base labels.
pub struct Labels<T> {
	base_labels: Arc<[(String, String)]>,
	ext: T,
}

impl Labels<()> {
	/// Create a new set of labels.
	pub fn new(base_labels: Vec<(String, String)>) -> Self {
		Self {
			base_labels: base_labels.into(),
			ext: (),
		}
	}
}

impl<T> Labels<T> {
	/// Extend the labels with additional labels.
	pub fn extend<Y>(&self, ext: Y) -> Labels<Y> {
		Labels {
			base_labels: self.base_labels.clone(),
			ext,
		}
	}
}

/// A custom implementation of EncodeLabelSet for Labels<T>.
impl<T: EncodeLabelSet> EncodeLabelSet for Labels<T> {
	fn encode(&self, mut encoder: prometheus_client::encoding::LabelSetEncoder) -> Result<(), std::fmt::Error> {
		for (key, value) in self.base_labels.iter() {
			let mut label_encoder = encoder.encode_label();
			let mut label_key_encoder = label_encoder.encode_label_key()?;
			EncodeLabelKey::encode(key, &mut label_key_encoder)?;
			let mut label_value_encoder = label_key_encoder.encode_label_value()?;
			EncodeLabelValue::encode(value, &mut label_value_encoder)?;
			label_value_encoder.finish()?;
		}

		self.ext.encode(encoder)
	}
}

pub trait MetricsProvider {
	fn ctx(&self) -> &scuffle_utils::context::Context;
	fn bind(&self) -> SocketAddr;
	fn registry(&self) -> &prometheus_client::registry::Registry;
	fn pre_hook(&self);
}

impl<M: MetricsProvider> MetricsProvider for Arc<M> {
	fn ctx(&self) -> &scuffle_utils::context::Context {
		(**self).ctx()
	}

	fn bind(&self) -> SocketAddr {
		(**self).bind()
	}

	fn registry(&self) -> &prometheus_client::registry::Registry {
		(**self).registry()
	}

	fn pre_hook(&self) {
		(**self).pre_hook()
	}
}

pub async fn run(provider: impl MetricsProvider) -> anyhow::Result<()> {
	let bind = provider.bind();
	let ctx = provider.ctx();
	tracing::info!("[metrics] listening on http://{}", bind);
	let socket = if bind.is_ipv6() {
		TcpSocket::new_v6()?
	} else {
		TcpSocket::new_v4()?
	};

	socket.set_reuseaddr(true).context("socket reuseaddr")?;
	socket.set_reuseport(true).context("socket reuseport")?;
	socket.bind(bind).context("socket bind")?;
	let listener = socket.listen(16)?;

	while let Ok(r) = listener.accept().context(ctx).await {
		let (socket, _) = r?;

		let registry = provider.registry();

		let provider = &provider;

		let service = service_fn(move |_| async {
			let mut body = String::new();

			provider.pre_hook();

			prometheus_client::encoding::text::encode(&mut body, registry).context("encode prometheus metrics")?;

			Ok::<_, anyhow::Error>({
				hyper::Response::builder()
					.header(hyper::header::CONTENT_TYPE, "text/plain")
					.status(StatusCode::OK)
					.body(Full::new(Bytes::from(body)))
					.context("build response")?
			})
		});

		http1::Builder::new()
			.serve_connection(TokioIo::new(socket), service)
			.timeout(Duration::from_secs(2))
			.await
			.ok();
	}

	Ok(())
}
