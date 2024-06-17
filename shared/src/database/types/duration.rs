#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "unit", content = "count")]
pub enum DurationUnit {
	Days(u32),
	Months(u32),
}
