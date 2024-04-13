use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
	T: Serialize,
	S: Serializer,
{
	match value {
		Some(value) => {
			let s = serde_json::to_string(value).map_err(|e| serde::ser::Error::custom(e))?;
			tracing::debug!("{}", s.len());
			serializer.serialize_str(&s)
		}
		None => serializer.serialize_none(),
	}
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
	T: for<'a> Deserialize<'a>,
	D: Deserializer<'de>,
{
	match Option::<String>::deserialize(deserializer)? {
		Some(s) => serde_json::from_str(&s).map_err(|e| serde::de::Error::custom(e)).map(Some),
		None => Ok(None),
	}
}
