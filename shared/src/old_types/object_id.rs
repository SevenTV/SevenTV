use std::fmt;
use std::str::FromStr;

use async_graphql::{InputValueError, Scalar, ScalarType};

use crate::database::{Badge, Emote, EmoteSet, Id, Paint, Role, Ticket, User, UserId};

pub type EmoteObjectId = ObjectId<Emote>;
pub type UserObjectId = ObjectId<User>;
pub type EmoteSetObjectId = ObjectId<EmoteSet>;
pub type TicketObjectId = ObjectId<Ticket>;
pub type RoleObjectId = ObjectId<Role>;
pub type PaintObjectId = ObjectId<Paint>;
pub type BadgeObjectId = ObjectId<Badge>;

// This is a workaround for backwards compatibility with the old ObjectId type

#[derive(Debug, Clone, Copy, Default)]
pub struct VirtualId(pub UserId);

#[derive(thiserror::Error, Debug)]
pub enum VirtualIdFromStrError {
	#[error("invalid user id: {0}")]
	UserId(#[from] crate::database::IdFromStrError),
	#[error("invalid virtual id")]
	InvalidVirtualId,
}

impl FromStr for VirtualId {
	type Err = VirtualIdFromStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.starts_with("V-") {
			UserId::from_str(s.trim_start_matches("V-")).map(Self).map_err(|e| e.into())
		} else {
			Err(VirtualIdFromStrError::InvalidVirtualId)
		}
	}
}

impl fmt::Display for VirtualId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "V-{}", self.0)
	}
}

impl serde::Serialize for VirtualId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.to_string().serialize(serializer)
	}
}

impl<'de> serde::Deserialize<'de> for VirtualId {
	fn deserialize<D>(deserializer: D) -> Result<VirtualId, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		VirtualId::from_str(&s).map_err(serde::de::Error::custom)
	}
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ObjectId<T> {
	VirtualId(VirtualId),
	Id(Id<T>),
}

impl<T> fmt::Debug for ObjectId<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::VirtualId(id) => fmt::Debug::fmt(id, f),
			Self::Id(id) => fmt::Debug::fmt(id, f),
		}
	}
}

impl<T> Clone for ObjectId<T> {
	fn clone(&self) -> Self {
		match self {
			Self::VirtualId(id) => Self::VirtualId(id.clone()),
			Self::Id(id) => Self::Id(id.clone()),
		}
	}
}

impl<T> Copy for ObjectId<T> {}

impl<T> Default for ObjectId<T> {
	fn default() -> Self {
		Self::Id(Id::default())
	}
}

impl<T> From<Id<T>> for ObjectId<T> {
	fn from(id: Id<T>) -> Self {
		Self::Id(id)
	}
}

impl<T> ObjectId<T> {
	pub fn id(self) -> Id<T> {
		match self {
			Self::VirtualId(id) => id.0.cast(),
			Self::Id(id) => id,
		}
	}
}

impl<T> FromStr for ObjectId<T> {
	type Err = VirtualIdFromStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if let Ok(v) = VirtualId::from_str(s) {
			return Ok(Self::VirtualId(v));
		}
		Id::<T>::from_str(s).map(Self::Id).map_err(Into::into)
	}
}

impl<T> fmt::Display for ObjectId<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::VirtualId(id) => fmt::Display::fmt(id, f),
			Self::Id(id) => fmt::Display::fmt(id, f),
		}
	}
}

#[Scalar(name = "ObjectID")]
impl<T: Sync + Send> ScalarType for ObjectId<T> {
	fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
		if let async_graphql::Value::String(s) = &value {
			if s.starts_with("V-") {
				return UserId::from_str(s.trim_start_matches("V-"))
					.map(|id| Self::VirtualId(VirtualId(id)))
					.map_err(|e| InputValueError::custom(e));
			}
		}
		Id::<T>::parse(value).map(Self::Id).map_err(|e| e.propagate())
	}

	fn to_value(&self) -> async_graphql::Value {
		match self {
			Self::VirtualId(id) => id.to_string().to_value(),
			Self::Id(id) => id.to_value(),
		}
	}
}
