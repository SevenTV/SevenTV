use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;

use super::{MetricsProvider, DEFAULT_HISTOGRAM_BUCKETS};

pub struct HttpMetrics {
	http_request_duration: Family<LabelsConnectionDuration, Histogram>,
}

impl HttpMetrics {
	pub fn observe_http_request_duration(&self, path: &'static str, method: &'static str, status: u16, duration: f64) {
		self.http_request_duration
			.get_or_create(&LabelsConnectionDuration::new(path, method, status))
			.observe(duration);
	}
}

impl Default for HttpMetrics {
	fn default() -> Self {
		let http_request_duration =
			Family::<_, _>::new_with_constructor(|| Histogram::new(DEFAULT_HISTOGRAM_BUCKETS.iter().copied()));

		Self { http_request_duration }
	}
}

impl MetricsProvider for HttpMetrics {
	fn register(&mut self, registry: &mut Registry) {
		registry.register(
			"http_request_duration_seconds",
			"The number of seconds used on http requests, by path, method and status",
			self.http_request_duration.clone(),
		);
	}
}

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, EncodeLabelSet)]
/// Memory labels.
struct LabelsConnectionDuration {
	path: &'static str,
	method: &'static str,
	status: u16,
}

impl LabelsConnectionDuration {
	fn new(path: &'static str, method: &'static str, status: u16) -> Self {
		Self { path, method, status }
	}
}
