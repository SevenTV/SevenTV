use std::marker::PhantomData;

#[derive(Debug)]
pub struct And<T>(Vec<Filter<T>>);
#[derive(Debug)]
pub struct Or<T>(Vec<Filter<T>>);
#[derive(Debug)]
pub struct Nor<T>(Vec<Filter<T>>);

#[doc(hidden)]
#[allow(private_bounds)]
pub trait FilterQueryHelper<T>: Sized + Into<Filter<T>> + Default + Sealed {
	fn merge_helper(self, items: impl IntoIterator<Item = Self>) -> Self;
}

macro_rules! impl_filter_structs {
	($variant:ident, $ty:ty, $key:literal) => {
		impl<T> Clone for $ty {
			fn clone(&self) -> Self {
				Self(self.0.clone())
			}
		}

		impl<T> $ty {
			pub fn new(value: impl IntoIterator<Item = Filter<T>>) -> Self {
				Self(value.into_iter().collect())
			}

			pub fn to_document(&self) -> bson::Document {
				bson::doc! { $key: &self.0 }
			}

			pub fn merge(self, items: impl IntoIterator<Item = Self>) -> Self {
				Self::new(std::iter::once(self).chain(items).flat_map(|doc| doc.0))
			}
		}

		impl<T> From<$ty> for bson::Bson {
			fn from(value: $ty) -> Self {
				value.to_document().into()
			}
		}

		impl<T> serde::Serialize for $ty {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: serde::Serializer,
			{
				self.to_document().serialize(serializer)
			}
		}

		impl<T> FilterQueryHelper<T> for $ty {
			fn merge_helper(self, items: impl IntoIterator<Item = Self>) -> Self {
				Self::merge(self, items)
			}
		}

		impl<T> Sealed for $ty {}

		impl<T> From<$ty> for Filter<T> {
			fn from(value: $ty) -> Self {
				Filter::$variant(value)
			}
		}

		impl<T> Default for $ty {
			fn default() -> Self {
				Self(vec![])
			}
		}
	};
}

impl_filter_structs!(And, And<T>, "$and");
impl_filter_structs!(Or, Or<T>, "$or");
impl_filter_structs!(Nor, Nor<T>, "$nor");

#[derive(Debug)]
pub enum Filter<T> {
	And(And<T>),
	Or(Or<T>),
	Nor(Nor<T>),
	Value(Value<T>),
}

impl<T> Filter<T> {
	pub fn to_document(&self) -> bson::Document {
		match self {
			Filter::And(value) => value.to_document(),
			Filter::Or(value) => value.to_document(),
			Filter::Nor(value) => value.to_document(),
			Filter::Value(value) => value.to_document(),
		}
	}

	pub fn merge<I: FilterQueryHelper<T>>(items: impl IntoIterator<Item = I>) -> I {
		let mut iter = items.into_iter();
		iter.next().unwrap_or_default().merge_helper(iter)
	}

	pub fn or<I: Into<Self>>(items: impl IntoIterator<Item = I>) -> Self {
		Self::Or(Or::new(items.into_iter().map(|doc| doc.into())))
	}

	pub fn nor<I: Into<Self>>(items: impl IntoIterator<Item = I>) -> Self {
		Self::Nor(Nor::new(items.into_iter().map(|doc| doc.into())))
	}

	pub fn and<I: Into<Self>>(items: impl IntoIterator<Item = I>) -> Self {
		Self::And(And::new(items.into_iter().map(|doc| doc.into())))
	}
}

impl<T> From<Filter<T>> for bson::Bson {
	fn from(value: Filter<T>) -> Self {
		match value {
			Filter::And(value) => value.into(),
			Filter::Or(value) => value.into(),
			Filter::Nor(value) => value.into(),
			Filter::Value(value) => value.into(),
		}
	}
}

impl<T> Clone for Filter<T> {
	fn clone(&self) -> Self {
		match self {
			Filter::And(value) => Filter::And(value.clone()),
			Filter::Or(value) => Filter::Or(value.clone()),
			Filter::Nor(value) => Filter::Nor(value.clone()),
			Filter::Value(value) => Filter::Value(value.clone()),
		}
	}
}

impl<T> serde::Serialize for Filter<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.to_document().serialize(serializer)
	}
}

#[derive(Debug)]
pub struct Value<T>(bson::Document, PhantomData<T>);

impl<T> serde::Serialize for Value<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.0.serialize(serializer)
	}
}

impl<T> Value<T> {
	pub fn new(doc: bson::Document) -> Self {
		Self(doc, PhantomData)
	}

	pub fn to_document(&self) -> bson::Document {
		self.0.clone()
	}

	pub fn merge(items: impl IntoIterator<Item = Self>) -> Self {
		Self::new(items.into_iter().flat_map(|doc| doc.0).collect())
	}
}

impl<T> Clone for Value<T> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), PhantomData)
	}
}

impl<T> From<Value<T>> for bson::Bson {
	fn from(value: Value<T>) -> Self {
		value.to_document().into()
	}
}

impl<T> Default for Value<T> {
	fn default() -> Self {
		Self(bson::Document::new(), PhantomData)
	}
}

impl<T> FilterQueryHelper<T> for Value<T> {
	fn merge_helper(self, items: impl IntoIterator<Item = Self>) -> Self {
		Self::merge(std::iter::once(self).chain(items))
	}
}

impl<T> Sealed for Value<T> {}

impl<T> From<Value<T>> for Filter<T> {
	fn from(value: Value<T>) -> Self {
		Filter::Value(value)
	}
}

#[doc(hidden)]
pub trait __AssertFilterBounds<T> {}
#[doc(hidden)]
pub trait __AssertFilterBoundsFlatten<T> {}

use super::traits::Sealed;
pub use super::traits::__ArrayLike;

/// This implies that for a filter of type `T`; `T` is a valid value as well as
/// `Value<T>`
impl<T> __AssertFilterBounds<T> for T {}
impl<T> __AssertFilterBounds<T> for &T {}
impl<T> __AssertFilterBoundsFlatten<T> for T {}
impl<T> __AssertFilterBoundsFlatten<T> for &T {}
impl<T> __AssertFilterBoundsFlatten<T> for Value<T> {}
impl<T> __AssertFilterBoundsFlatten<T> for &Value<T> {}

impl<T> __AssertFilterBounds<Option<T>> for T {}
impl<T> __AssertFilterBounds<Option<T>> for &T {}
impl<T> __AssertFilterBounds<Option<T>> for Option<&T> {}

impl<T> __AssertFilterBounds<Vec<T>> for T {}
impl<T> __AssertFilterBounds<Vec<T>> for &T {}
impl<T> __AssertFilterBounds<Vec<T>> for &[T] {}
impl<T> __AssertFilterBoundsFlatten<Vec<T>> for T {}
impl<T> __AssertFilterBoundsFlatten<Vec<T>> for &T {}
impl<T> __AssertFilterBoundsFlatten<Vec<T>> for &[T] {}
impl<T> __AssertFilterBoundsFlatten<Vec<T>> for Value<T> {}
impl<T> __AssertFilterBoundsFlatten<Vec<T>> for &Value<T> {}

pub use macros::mongo_filter_query as filter;
