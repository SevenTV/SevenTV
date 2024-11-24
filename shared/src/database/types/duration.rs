#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "unit", content = "count")]
pub enum DurationUnit {
	Days(i32),
	Months(i32),
}

impl DurationUnit {
	pub fn estimate_days(&self) -> f64 {
		match self {
			Self::Days(days) => *days as f64,
			Self::Months(months) => *months as f64 * 30.44,
		}
	}
}

impl PartialEq for DurationUnit {
	fn eq(&self, other: &Self) -> bool {
		self.cmp(other) == std::cmp::Ordering::Equal
	}
}

impl Eq for DurationUnit {}

impl PartialOrd for DurationUnit {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for DurationUnit {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.estimate_days().total_cmp(&other.estimate_days())
	}
}
