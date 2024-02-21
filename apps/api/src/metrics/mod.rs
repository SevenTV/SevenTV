use memory_stats::memory_stats;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use shared::metrics::Labels;

use self::labels::Memory;

mod labels;

pub struct Metrics {
	registry: Registry,
	memory: Family<Labels<Memory>, Gauge>,
	labels: Labels<()>,
}

impl Metrics {
	pub fn new(mut labels: Vec<(String, String)>) -> Self {
		let mut registry = Registry::default();

		let memory = Family::default();

		labels.push(("version".into(), env!("CARGO_PKG_VERSION").into()));

		registry.register("api_memory_bytes", "The amount of memory used", memory.clone());

		Self {
			registry,
			memory,
			labels: Labels::new(labels),
		}
	}

	/// Observe memory usage.
	pub fn observe_memory(&self) {
		self.memory
			.get_or_create(&self.labels.extend(Memory::ALLOCATED))
			.set(super::ALLOCATOR.allocated() as i64);
		self.memory
			.get_or_create(&self.labels.extend(Memory::REMAINING))
			.set(super::ALLOCATOR.remaining() as i64);

		if let Some(usage) = memory_stats() {
			self.memory
				.get_or_create(&self.labels.extend(Memory::RESIDENT))
				.set(usage.physical_mem as i64);
			self.memory
				.get_or_create(&self.labels.extend(Memory::VIRTUAL))
				.set(usage.virtual_mem as i64);
		} else {
			tracing::warn!("failed to get memory stats");
		}
	}

	pub fn registry(&self) -> &Registry {
		&self.registry
	}
}
