use prometheus_client::encoding::EncodeLabelSet;

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, EncodeLabelSet)]
/// Memory labels.
pub struct Memory {
	kind: &'static str,
}

impl Memory {
	/// Allocated memory.
	pub const ALLOCATED: Self = Self { kind: "allocated" };
	/// Free memory.
	pub const REMAINING: Self = Self { kind: "remaining" };
	/// Virtual memory.
	pub const RESIDENT: Self = Self { kind: "resident" };
	/// Virtual memory.
	pub const VIRTUAL: Self = Self { kind: "virtual" };
}
