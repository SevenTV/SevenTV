use bson::DateTime as BsonDateTime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub trait SerdeHelper: Sized + serde::de::DeserializeOwned {
	type BsonType: serde::Serialize + serde::de::DeserializeOwned;

	fn into_bson(&self) -> Self::BsonType;
	fn from_bson(bson: Self::BsonType) -> Self;
}

impl SerdeHelper for DateTime<Utc> {
	type BsonType = BsonDateTime;

	fn into_bson(&self) -> Self::BsonType {
		BsonDateTime::from_chrono(*self)
	}

	fn from_bson(bson: Self::BsonType) -> Self {
		bson.into()
	}
}

// impl SerdeHelper for Option<DateTime<Utc>> {
// 	type BsonType = Option<BsonDateTime>;

// 	fn into_bson(&self) -> Self::BsonType {
// 		self.map(BsonDateTime::from_chrono)
// 	}

// 	fn from_bson(bson: Self::BsonType) -> Self {
// 		bson.map(Into::into)
// 	}
// }

impl<T: SerdeHelper> SerdeHelper for Option<T> {
	type BsonType = Option<T::BsonType>;

	fn into_bson(&self) -> Self::BsonType {
		self.as_ref().map(|t| t.into_bson())
	}

	fn from_bson(bson: Self::BsonType) -> Self {
		bson.map(|t| T::from_bson(t))
	}
}

impl<T: SerdeHelper> SerdeHelper for Vec<T> {
	type BsonType = Vec<T::BsonType>;

	fn into_bson(&self) -> Self::BsonType {
		self.iter().map(|t| t.into_bson()).collect()
	}

	fn from_bson(bson: Self::BsonType) -> Self {
		bson.into_iter().map(|t| T::from_bson(t)).collect()
	}
}

pub fn serialize<S, C: SerdeHelper>(input: &C, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	input.into_bson().serialize(serializer)
}

pub fn deserialize<'de, D, C: SerdeHelper>(deserializer: D) -> Result<C, D::Error>
where
	D: Deserializer<'de>,
{
	let bson = C::BsonType::deserialize(deserializer)?;
	Ok(C::from_bson(bson))
}
