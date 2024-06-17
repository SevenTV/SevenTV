use std::fmt;
use std::str::FromStr;

use async_graphql::{Scalar, ScalarType};

use crate::database::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GqlObjectId(pub Id<()>);

impl<T> From<Id<T>> for GqlObjectId {
	fn from(id: Id<T>) -> Self {
		Self(id.cast())
	}
}

impl serde::Serialize for GqlObjectId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.0.serialize(serializer)
	}
}

impl<'de> serde::Deserialize<'de> for GqlObjectId {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Id::<()>::deserialize(deserializer).map(Self)
	}
}

impl FromStr for GqlObjectId {
	type Err = <Id<()> as FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Id::<()>::from_str(s).map(Self).map_err(Into::into)
	}
}

impl fmt::Display for GqlObjectId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

#[Scalar(name = "ObjectID")]
impl ScalarType for GqlObjectId {
	fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
		Id::<()>::parse(value).map(Self).map_err(|e| e.propagate())
	}

	fn to_value(&self) -> async_graphql::Value {
		self.0.to_value()
	}
}
