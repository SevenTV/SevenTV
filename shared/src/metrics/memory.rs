use memory_stats::memory_stats;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;

use super::MetricsProvider;

#[derive(Default)]
pub struct MemoryMetrics {
	memory: Family<LabelsMemory, Gauge>,
	allocator_fn: Option<fn() -> (usize, usize)>,
}

impl MemoryMetrics {
	pub fn new(allocator_fn: fn() -> (usize, usize)) -> Self {
		Self {
			memory: Default::default(),
			allocator_fn: Some(allocator_fn),
		}
	}
}

impl MemoryMetrics {
	/// Observe memory usage.
	pub fn observe_memory(&self) {
		if let Some(allocator_fn) = self.allocator_fn {
			let (allocated, remaining) = allocator_fn();

			self.memory.get_or_create(&LabelsMemory::ALLOCATED).set(allocated as i64);
			self.memory.get_or_create(&LabelsMemory::REMAINING).set(remaining as i64);
		}

		if let Some(usage) = memory_stats() {
			self.memory
				.get_or_create(&LabelsMemory::RESIDENT)
				.set(usage.physical_mem as i64);
			self.memory
				.get_or_create(&LabelsMemory::VIRTUAL)
				.set(usage.virtual_mem as i64);
		}
	}
}

impl MetricsProvider for MemoryMetrics {
	fn register(&mut self, registry: &mut prometheus_client::registry::Registry) {
		registry.register("rust_memory", "Memory usage", self.memory.clone());
	}

	fn pre_encode(&self) {
		self.observe_memory();
	}
}

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, EncodeLabelSet)]
/// Memory labels.
struct LabelsMemory {
	kind: &'static str,
}

impl LabelsMemory {
	/// Allocated memory.
	pub const ALLOCATED: Self = Self { kind: "allocated" };
	/// Free memory.
	pub const REMAINING: Self = Self { kind: "remaining" };
	/// Virtual memory.
	pub const RESIDENT: Self = Self { kind: "resident" };
	/// Virtual memory.
	pub const VIRTUAL: Self = Self { kind: "virtual" };
}
