use std::fmt;

use chrono::TimeZone;
use mongodb::bson;

pub struct Id<S = ()>(ulid::Ulid, std::marker::PhantomData<S>);

impl<S> fmt::Debug for Id<S> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple(std::any::type_name::<Self>()).field(&self.0).finish()
	}
}

impl<S> Clone for Id<S> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<S> Copy for Id<S> {}

impl<S> PartialEq for Id<S> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl<S> Eq for Id<S> {}

impl<S> std::hash::Hash for Id<S> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.0.hash(state)
	}
}

impl<S> PartialOrd for Id<S> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.0.cmp(&other.0))
	}
}

impl<S> Ord for Id<S> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<S> Id<S> {
	pub fn new() -> Self {
		Self::from_ulid(ulid::Ulid::new())
	}

	pub fn with_timestamp_ms(timestamp_ms: u64) -> Self {
		Self::with_timestamp(chrono::Utc.timestamp_millis_opt(timestamp_ms as i64).unwrap())
	}

	pub fn with_timestamp(timestamp: chrono::DateTime<chrono::Utc>) -> Self {
		Self::from_ulid(ulid::Ulid::from_datetime(timestamp.into()))
	}

	pub const fn as_ulid(&self) -> ulid::Ulid {
		self.0
	}

	pub const fn as_uuid(&self) -> uuid::Uuid {
		uuid::Uuid::from_bytes(self.0.to_bytes())
	}

	pub const fn is_object_id_compatible(&self) -> bool {
		self.timestamp_ms() % 1000 == 0 // MongoDB ObjectId has a timestamp resolution of 1 second
			&& self.random() <= u64::MAX as u128 // MongoDB ObjectId has a random value of 8 bytes
			&& self.timestamp_ms() / 1000 <= u32::MAX as u64 // MongoDB ObjectId has a timestamp value of 4 bytes
	}

	pub const fn as_object_id(&self) -> Option<bson::oid::ObjectId> {
		if !self.is_object_id_compatible() {
			None
		} else {
			let timestamp = ((self.timestamp_ms() / 1000) as u32).to_be_bytes();
			let random = (self.random() as u64).to_be_bytes();

			Some(bson::oid::ObjectId::from_bytes([
				timestamp[0],
				timestamp[1],
				timestamp[2],
				timestamp[3],
				random[0],
				random[1],
				random[2],
				random[3],
				random[4],
				random[5],
				random[6],
				random[7],
			]))
		}
	}

	pub const fn is_nil(&self) -> bool {
		self.0.is_nil()
	}

	pub const fn timestamp_ms(&self) -> u64 {
		self.0.timestamp_ms()
	}

	pub fn datetime(&self) -> chrono::DateTime<chrono::Utc> {
		self.0.datetime().into()
	}

	pub const fn random(&self) -> u128 {
		self.0.random()
	}

	pub const fn nil() -> Self {
		Self::from_ulid(ulid::Ulid::nil())
	}

	pub const fn from_ulid(ulid: ulid::Ulid) -> Self {
		Self(ulid, std::marker::PhantomData)
	}

	pub const fn from_uuid(uuid: uuid::Uuid) -> Self {
		Self::from_ulid(ulid::Ulid::from_bytes(uuid.into_bytes()))
	}

	pub const fn from_bytes(bytes: [u8; 16]) -> Self {
		Self::from_ulid(ulid::Ulid::from_bytes(bytes))
	}

	pub fn into_bytes(self) -> [u8; 16] {
		self.0.to_bytes()
	}

	pub const fn from_object_id(object_id: bson::oid::ObjectId) -> Self {
		let bytes = object_id.bytes();

		let timestamp = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

		let random = u64::from_be_bytes([
			bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11],
		]);

		Self::from_parts(timestamp as u64, random as u128)
	}

	pub const fn from_parts(timestamp: u64, random: u128) -> Self {
		Self::from_ulid(ulid::Ulid::from_parts(timestamp, random))
	}

	pub const fn cast<T>(self) -> Id<T> {
		Id::from_ulid(self.0)
	}
}

impl<S> From<ulid::Ulid> for Id<S> {
	fn from(ulid: ulid::Ulid) -> Self {
		Self::from_ulid(ulid)
	}
}

impl<S> From<uuid::Uuid> for Id<S> {
	fn from(uuid: uuid::Uuid) -> Self {
		Self::from_uuid(uuid)
	}
}

impl<S> From<bson::uuid::Uuid> for Id<S> {
	fn from(uuid: bson::uuid::Uuid) -> Self {
		Self::from_bytes(uuid.bytes())
	}
}

impl<S> From<bson::oid::ObjectId> for Id<S> {
	fn from(object_id: bson::oid::ObjectId) -> Self {
		Self::from_object_id(object_id)
	}
}

impl<S> From<Id<S>> for ulid::Ulid {
	fn from(id: Id<S>) -> ulid::Ulid {
		id.0
	}
}

impl<S> From<Id<S>> for uuid::Uuid {
	fn from(id: Id<S>) -> uuid::Uuid {
		uuid::Uuid::from_bytes(id.0.to_bytes())
	}
}

impl<S> From<Id<S>> for bson::uuid::Uuid {
	fn from(id: Id<S>) -> bson::uuid::Uuid {
		bson::uuid::Uuid::from_bytes(id.0.to_bytes())
	}
}

impl<S> From<Id<S>> for bson::Bson {
	fn from(id: Id<S>) -> bson::Bson {
		bson::uuid::Uuid::from(id).into()
	}
}

impl<S> Default for Id<S> {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum IdFromStrError {
	#[error("invalid ulid: {0}")]
	Ulid(ulid::DecodeError),
	#[error("invalid uuid: {0}")]
	Uuid(uuid::Error),
	#[error("invalid object id: {0}")]
	ObjectId(bson::oid::Error),
	#[error("invalid id length: {0}")]
	InvalidLength(usize),
}

impl<S> std::str::FromStr for Id<S> {
	type Err = IdFromStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.len() {
			26 => s.parse().map_err(IdFromStrError::Ulid).map(Self::from_ulid),
			32 => s.parse().map_err(IdFromStrError::Uuid).map(Self::from_uuid),
			24 => s.parse().map_err(IdFromStrError::ObjectId).map(Self::from_object_id),
			len => Err(IdFromStrError::InvalidLength(len)),
		}
	}
}

impl<S> std::fmt::Display for Id<S> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl<S> serde::Serialize for Id<S> {
	fn serialize<T: serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
		// This is a hack to check if the serializer is a bson serializer because
		// bson::uuid::Uuid has a custom implementation for bson serialization
		let serializer_name = std::any::type_name::<T>()
			.trim_start_matches('&')
			.trim_start_matches("mut")
			.trim();
		if serializer_name.starts_with("bson::") {
			bson::uuid::Uuid::from(*self).serialize(serializer)
		} else {
			self.0.serialize(serializer)
		}
	}
}

impl<'de, S> serde::Deserialize<'de> for Id<S> {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		// This is a hack to check if the deserializer is a bson deserializer because
		// bson::uuid::Uuid has a custom implementation for bson deserialization
		let deserializer_name = std::any::type_name::<D>()
			.trim_start_matches('&')
			.trim_start_matches("mut")
			.trim();
		if deserializer_name.starts_with("bson::") {
			bson::uuid::Uuid::deserialize(deserializer).map(Self::from)
		} else {
			// Parse the string
			let s = String::deserialize(deserializer)?;
			s.parse().map_err(serde::de::Error::custom)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_bson_serde() {
		// bson serialization
		let id = Id::new();

		assert_eq!(bson::to_bson(&id).unwrap(), bson::Bson::from(bson::uuid::Uuid::from(id)));

		let returned: Id = bson::from_bson(bson::Bson::from(bson::uuid::Uuid::from(id))).unwrap();

		assert_eq!(id, returned);
	}

	#[test]
	fn test_json_serde() {
		// json serialization
		let id = Id::new();

		let json = serde_json::to_string(&id).unwrap();

		assert_eq!(json, format!("\"{}\"", id));

		let returned: Id = serde_json::from_str(&json).unwrap();

		assert_eq!(id, returned);
	}
}
