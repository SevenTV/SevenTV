#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "unit", content = "count")]
pub enum DurationUnit {
	Days(i32),
	Months(i32),
}
