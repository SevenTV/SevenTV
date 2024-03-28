#![allow(unused)]

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, utoipa::ToSchema)]
/// MongoDB ObjectIDs are 12-byte BSON strings, representing a 4-byte timestamp,
/// 5-byte random value, and a 3-byte incrementing counter. https://docs.mongodb.com/manual/reference/method/ObjectId/
pub struct ObjectId(u128);

impl Default for ObjectId {
	fn default() -> Self {
		Self::empty()
	}
}

impl ObjectId {
	/// Create a new empty ObjectID.
	pub const fn empty() -> Self {
		Self(0)
	}

	/// Get the timestamp of the ObjectID.
	pub const fn timestamp(self) -> u64 {
		(self.0 >> 64) as u64
	}

	/// Get the random value of the ObjectID.
	pub const fn random(self) -> u64 {
		self.0 as u64
	}

	/// Convert the ObjectID into a ULID.
	pub const fn into_ulid(self) -> ulid::Ulid {
		ulid::Ulid::from_parts(self.timestamp() * 1000, self.random() as u128)
	}

	/// Create a new ObjectID from the given timestamp and random value.
	pub const fn from_parts(timestamp: u64, random: u64) -> Self {
		Self((timestamp as u128) << 64 | random as u128)
	}

	/// Create a new ObjectID from the given ULID, this will truncate the
	/// timestamp to seconds.
	pub const fn from_ulid(ulid: ulid::Ulid) -> Self {
		Self::from_parts(ulid.timestamp_ms() / 1000, ulid.random() as u64)
	}
}

impl From<ulid::Ulid> for ObjectId {
	fn from(ulid: ulid::Ulid) -> Self {
		Self::from_ulid(ulid)
	}
}

impl From<ObjectId> for ulid::Ulid {
	fn from(id: ObjectId) -> Self {
		id.into_ulid()
	}
}

impl std::fmt::Display for ObjectId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let data = self.0.to_be_bytes();
		f.write_str(&hex::encode(&data[4..]))
	}
}

impl std::str::FromStr for ObjectId {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut data = [0u8; 16];
		hex::decode_to_slice(s, &mut data[4..]).map_err(|err| {
			tracing::warn!("failed to decode ObjectID: {err} - {s}");
		})?;

		Ok(ObjectId(u128::from_be_bytes(data)))
	}
}

impl serde::Serialize for ObjectId {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.collect_str(self)
	}
}

impl<'a> serde::Deserialize<'a> for ObjectId {
	fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_str(ObjectIDVisitor)
	}
}

/// A custom visitor for ObjectID.
struct ObjectIDVisitor;

impl<'a> serde::de::Visitor<'a> for ObjectIDVisitor {
	type Value = ObjectId;

	fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		formatter.write_str("ObjectID")
	}

	fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
		ObjectId::from_str(value).map_err(|_| E::custom("invalid ObjectID"))
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use super::*;

	#[test]
	fn test_object_id() {
		let id = ObjectId::from_str("00000020f51bb4362eee2a4d").unwrap();
		assert_eq!(id.timestamp(), 0x00000020);
		assert_eq!(id.random(), 0xf51bb4362eee2a4d);
		assert_eq!(id.to_string(), "00000020f51bb4362eee2a4d");

		let ulid = id.into_ulid();

		assert_eq!(ulid.to_string(), "0000000Z80000FA6XM6RQEWAJD");

		let id = ObjectId::from_ulid(ulid);
		assert_eq!(id.timestamp(), 0x00000020);
		assert_eq!(id.random(), 0xf51bb4362eee2a4d);
	}
}
