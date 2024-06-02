use core::fmt;
use std::str::FromStr;

use async_graphql::{InputValueError, Scalar, ScalarType};
use shared::database::{Badge, Emote, EmoteSet, Id, Paint, Role, Ticket, User, UserId};

pub type EmoteObjectId = ObjectId<Emote>;
pub type UserObjectId = ObjectId<User>;
pub type EmoteSetObjectId = ObjectId<EmoteSet>;
pub type TicketObjectId = ObjectId<Ticket>;
pub type RoleObjectId = ObjectId<Role>;
pub type PaintObjectId = ObjectId<Paint>;
pub type BadgeObjectId = ObjectId<Badge>;

// This is a workaround for backwards compatibility with the old ObjectId type

pub enum ObjectId<T> {
	VirtualId(UserId),
	Id(Id<T>),
}

impl<T> fmt::Debug for ObjectId<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::VirtualId(id) => id.fmt(f),
			Self::Id(id) => id.fmt(f),
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
			Self::VirtualId(id) => id.cast(),
			Self::Id(id) => id,
		}
	}
}

#[Scalar(name = "ObjectID")]
impl<T: Sync + Send> ScalarType for ObjectId<T> {
	fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
		if let async_graphql::Value::String(s) = &value {
			if s.starts_with("V-") {
				return UserId::from_str(s.trim_start_matches("V-"))
					.map(Self::VirtualId)
					.map_err(|e| InputValueError::custom(e));
			}
		}
		Id::<T>::parse(value).map(Self::Id).map_err(|e| e.propagate())
	}

	fn to_value(&self) -> async_graphql::Value {
		match self {
			Self::VirtualId(id) => format!("V-{}", id).to_value(),
			Self::Id(id) => id.to_value(),
		}
	}
}
