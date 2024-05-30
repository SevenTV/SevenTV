use core::fmt;
use std::ops::Deref;

use async_graphql::{Scalar, ScalarType};
use shared::database::{Badge, Emote, EmoteSet, Id, Paint, Role, Ticket, User};

pub type EmoteObjectId = ObjectId<Emote>;
pub type UserObjectId = ObjectId<User>;
pub type EmoteSetObjectId = ObjectId<EmoteSet>;
pub type TicketObjectId = ObjectId<Ticket>;
pub type RoleObjectId = ObjectId<Role>;
pub type PaintObjectId = ObjectId<Paint>;
pub type BadgeObjectId = ObjectId<Badge>;

// This is a workaround for backwards compatibility with the old ObjectId type

pub struct ObjectId<T>(Id<T>);

impl<T> fmt::Debug for ObjectId<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl<T> Clone for ObjectId<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Copy for ObjectId<T> {}

impl<T> Default for ObjectId<T> {
    fn default() -> Self {
        Self(Id::default())
    }
}

impl<T> From<Id<T>> for ObjectId<T> {
    fn from(id: Id<T>) -> Self {
        Self(id)
    }
}

impl<T> From<ObjectId<T>> for Id<T> {
    fn from(object_id: ObjectId<T>) -> Self {
        object_id.0
    }
}

impl<T> Deref for ObjectId<T> {
    type Target = Id<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ObjectId<T> {
    pub fn real(self) -> Id<T> {
        self.0
    }
}

#[Scalar]
impl<T: Sync + Send> ScalarType for ObjectId<T> {
	fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
		Id::<T>::parse(value).map(ObjectId).map_err(|e| e.propagate())
	}

	fn to_value(&self) -> async_graphql::Value {
		self.0.to_value()
	}
}
