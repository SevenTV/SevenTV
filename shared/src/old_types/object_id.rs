use std::{fmt, ops::Deref};
use std::str::FromStr;

use async_graphql::{Scalar, ScalarType};

use crate::database::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GqlObjectId {
	pub id: Id<()>,
	pub old: bool,
}

impl Deref for GqlObjectId {
	type Target = Id<()>;

	fn deref(&self) -> &Self::Target {
		&self.id
	}
}

impl GqlObjectId {
	pub fn new(id: Id<()>) -> Self {
		Self { id, old: false }
	}

	pub fn old(id: Id<()>) -> Self {
		Self { id, old: true }
	}

	pub fn id<T>(self) -> Id<T> {
		self.id.cast()
	}
}

impl<T> From<Id<T>> for GqlObjectId {
	fn from(id: Id<T>) -> Self {
		Self {
			id: id.cast(),
			old: false,
		}
	}
}

impl serde::Serialize for GqlObjectId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		if self.old {
			if let Some(id) = self.id.as_object_id() {
				return id.serialize(serializer);
			}
		}

		self.id.serialize(serializer)
	}
}

impl<'de> serde::Deserialize<'de> for GqlObjectId {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Id::<()>::deserialize(deserializer).map(Self::new)
	}
}

impl FromStr for GqlObjectId {
	type Err = <Id<()> as FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Id::<()>::from_str(s).map(Self::new).map_err(Into::into)
	}
}

impl fmt::Display for GqlObjectId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.id.fmt(f)
	}
}

#[Scalar(name = "ObjectID")]
impl ScalarType for GqlObjectId {
	fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
		Id::<()>::parse(value).map(Self::new).map_err(|e| e.propagate())
	}

	fn to_value(&self) -> async_graphql::Value {
		if self.old {
			if let Some(id) = self.id.as_object_id() {
				return async_graphql::Value::String(id.to_string());
			}
		}

		self.id.to_value()
	}
}
