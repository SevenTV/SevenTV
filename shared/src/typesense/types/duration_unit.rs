use super::impl_typesense_type;
use crate::database;

#[derive(
	Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, serde_repr::Serialize_repr, serde_repr::Deserialize_repr,
)]
#[repr(i32)]
pub enum DurationUnit {
	Days = 0,
	Months = 1,
}

impl DurationUnit {
	pub fn split(value: database::duration::DurationUnit) -> (DurationUnit, i32) {
		match value {
			database::duration::DurationUnit::Days(x) => (DurationUnit::Days, x),
			database::duration::DurationUnit::Months(x) => (DurationUnit::Months, x),
		}
	}
}

impl_typesense_type!(DurationUnit, Int32);
