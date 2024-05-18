use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod optional;

pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
	T: Serialize,
	S: Serializer,
{
	let s = serde_json::to_string(value).map_err(serde::ser::Error::custom)?;
	serializer.serialize_str(&s)
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
	T: for<'a> Deserialize<'a>,
	D: Deserializer<'de>,
{
	let s = String::deserialize(deserializer)?;
	serde_json::from_str(&s).map_err(serde::de::Error::custom)
}
