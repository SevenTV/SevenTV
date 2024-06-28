#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "unit", content = "count")]
pub enum DurationUnit {
	Days(u64),
	Months(u64),
}
