#[derive(async_graphql::SimpleObject)]
pub struct TimePeriod {
	pub start: chrono::DateTime<chrono::Utc>,
	pub end: chrono::DateTime<chrono::Utc>,
}

impl From<shared::database::product::TimePeriod> for TimePeriod {
	fn from(value: shared::database::product::TimePeriod) -> Self {
		Self {
			start: value.start,
			end: value.end,
		}
	}
}

#[derive(async_graphql::InputObject)]
pub struct TimePeriodInput {
	pub start: chrono::DateTime<chrono::Utc>,
	pub end: chrono::DateTime<chrono::Utc>,
}

impl From<TimePeriodInput> for shared::database::product::TimePeriod {
	fn from(value: TimePeriodInput) -> Self {
		Self {
			start: value.start,
			end: value.end,
		}
	}
}
