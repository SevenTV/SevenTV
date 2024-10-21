use std::marker::PhantomData;

macro_rules! impl_update_structs {
    (
        $([$name:ident, $key:literal, $tyi:ident, $ty:ty]),*
    ) => {
        #[derive(Debug, typed_builder::TypedBuilder, serde::Serialize)]
        #[builder(field_defaults(default, setter(into)))]
        pub struct Update<T> {
            $(
                #[serde(skip_serializing_if = "Option::is_none", rename = $key)]
                #[serde(bound(serialize = ""))]
                pub $name: Option<$ty>,
            )*
        }

        impl<T> Update<T> {
            pub fn extend<I: Into<Self>>(mut self, items: impl IntoIterator<Item = I>) -> Self {
                for item in items {
                    let item = item.into();
                    $(
                        if let Some(v) = self.$name.take() {
                            self.$name = Some(v.merge(item.$name));
                        } else {
                            self.$name = item.$name;
                        }
                    )*
                }

                self
            }

            pub fn extend_one(self, item: impl Into<Self>) -> Self {
                self.extend(std::iter::once(item))
            }
        }

        impl<T> From<Update<T>> for bson::Bson {
            fn from(value: Update<T>) -> Self {
                value.to_document().into()
            }
        }

        impl<T> Clone for Update<T> {
            fn clone(&self) -> Self {
                Self {
                    $(
                        $name: self.$name.clone(),
                    )*
                }
            }
        }

        impl<T> Default for Update<T> {
            fn default() -> Self {
                Self {
                    $(
                        $name: None,
                    )*
                }
            }
        }

        $(
            #[derive(Debug)]
            pub struct $tyi<T>(bson::Document, PhantomData<T>);

            impl<T> Clone for $ty {
                fn clone(&self) -> Self {
                    Self(self.0.clone(), PhantomData)
                }
            }

            impl<T> serde::Serialize for $ty {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    self.0.serialize(serializer)
                }
            }

            impl<T> From<$ty> for bson::Bson {
                fn from(value: $ty) -> Self {
                    value.0.into()
                }
            }

            impl<T> From<$ty> for Update<T> {
                fn from(value: $ty) -> Self {
                    Self {
                        $name: Some(value),
                        ..Default::default()
                    }
                }
            }

            impl<T> $ty {
                pub fn new(doc: bson::Document) -> Self {
                    Self(doc, PhantomData)
                }

                pub fn to_document(&self) -> bson::Document {
                    bson::doc! { $key: &self.0 }
                }

                pub fn merge(self, items: impl IntoIterator<Item = Self>) -> Self {
                    Self::new(std::iter::once(self).chain(items).flat_map(|doc| doc.0).collect())
                }
            }
        )*
    };
}

impl_update_structs! {
	[set, "$set", Set, Set<T>],
	[unset, "$unset", Unset, Unset<T>],
	[inc, "$inc", Inc, Inc<T>],
	[mul, "$mul", Mul, Mul<T>],
	[max, "$max", Max, Max<T>],
	[min, "$min", Min, Min<T>],
	[push, "$push", Push, Push<T>],
	[pull, "$pull", Pull, Pull<T>],
	[add_to_set, "$addToSet", AddToSet, AddToSet<T>],
	[pop, "$pop", Pop, Pop<T>],
	[pull_all, "$pullAll", PullAll, PullAll<T>],
	[bit, "$bit", Bit, Bit<T>],
	[set_on_insert, "$setOnInsert", SetOnInsert, SetOnInsert<T>]
}

impl<T> Update<T> {
	pub fn to_document(&self) -> bson::Document {
		bson::to_document(&self).expect("Failed to serialize")
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayIndex {
	Int(usize),
	DollarSign,
}

impl std::fmt::Display for ArrayIndex {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Int(value) => write!(f, "{value}"),
			Self::DollarSign => write!(f, "$"),
		}
	}
}

impl From<usize> for ArrayIndex {
	fn from(value: usize) -> Self {
		Self::Int(value)
	}
}

pub trait __AssertUpdateBounds<U, T> {}

pub trait __AssertUpdateBoundsFlatten<U, T> {}

use super::filter;
pub use super::traits::__ArrayLike;

impl<T> __AssertUpdateBounds<Set<T>, T> for T {}
impl<T> __AssertUpdateBounds<Set<T>, T> for &T {}
impl<T> __AssertUpdateBounds<Set<Option<T>>, Option<T>> for T {}
impl<T> __AssertUpdateBounds<Set<Option<T>>, Option<T>> for &T {}
impl<T> __AssertUpdateBounds<Set<Option<T>>, Option<T>> for Option<&T> {}
impl<T> __AssertUpdateBoundsFlatten<Set<T>, T> for T {}
impl<T> __AssertUpdateBoundsFlatten<Set<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Set<T>, T> for Set<T> {}
impl<T> __AssertUpdateBoundsFlatten<Set<T>, T> for &Set<T> {}
impl<T> __AssertUpdateBoundsFlatten<Set<Option<T>>, Option<T>> for Set<T> {}
impl<T> __AssertUpdateBoundsFlatten<Set<Option<T>>, Option<T>> for &Set<T> {}

impl<T> __AssertUpdateBounds<Unset<T>, T> for bool {}
impl<T> __AssertUpdateBoundsFlatten<Unset<T>, T> for bool {}
impl<T> __AssertUpdateBoundsFlatten<Unset<T>, T> for Unset<T> {}
impl<T> __AssertUpdateBoundsFlatten<Unset<T>, T> for &Unset<T> {}

impl<T> __AssertUpdateBounds<Inc<T>, T> for T {}
impl<T> __AssertUpdateBounds<Inc<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Inc<T>, T> for T {}
impl<T> __AssertUpdateBoundsFlatten<Inc<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Inc<T>, T> for Inc<T> {}
impl<T> __AssertUpdateBoundsFlatten<Inc<T>, T> for &Inc<T> {}

impl<T> __AssertUpdateBounds<Mul<T>, T> for T {}
impl<T> __AssertUpdateBounds<Mul<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Mul<T>, T> for T {}
impl<T> __AssertUpdateBoundsFlatten<Mul<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Mul<T>, T> for Mul<T> {}
impl<T> __AssertUpdateBoundsFlatten<Mul<T>, T> for &Mul<T> {}

impl<T> __AssertUpdateBounds<Max<T>, T> for T {}
impl<T> __AssertUpdateBounds<Max<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Max<T>, T> for T {}
impl<T> __AssertUpdateBoundsFlatten<Max<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Max<T>, T> for Max<T> {}
impl<T> __AssertUpdateBoundsFlatten<Max<T>, T> for &Max<T> {}

impl<T> __AssertUpdateBounds<Min<T>, T> for T {}
impl<T> __AssertUpdateBounds<Min<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Min<T>, T> for T {}
impl<T> __AssertUpdateBoundsFlatten<Min<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Min<T>, T> for Min<T> {}
impl<T> __AssertUpdateBoundsFlatten<Min<T>, T> for &Min<T> {}

impl<T> __AssertUpdateBounds<Push<T>, T> for T {}
impl<T> __AssertUpdateBounds<Push<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Push<T>, T> for T {}
impl<T> __AssertUpdateBoundsFlatten<Push<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Push<T>, T> for Push<T> {}
impl<T> __AssertUpdateBoundsFlatten<Push<T>, T> for &Push<T> {}

impl<T> __AssertUpdateBounds<AddToSet<T>, T> for T {}
impl<T> __AssertUpdateBounds<AddToSet<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<AddToSet<T>, T> for T {}
impl<T> __AssertUpdateBoundsFlatten<AddToSet<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<AddToSet<T>, T> for AddToSet<T> {}
impl<T> __AssertUpdateBoundsFlatten<AddToSet<T>, T> for &AddToSet<T> {}

impl<T> __AssertUpdateBounds<Pull<T>, T> for T {}
impl<T> __AssertUpdateBounds<Pull<T>, T> for &T {}
impl<T> __AssertUpdateBounds<Pull<T>, T> for filter::Value<T> {}
impl<T> __AssertUpdateBounds<Pull<T>, T> for &filter::Value<T> {}
impl<T> __AssertUpdateBoundsFlatten<Pull<T>, T> for T {}
impl<T> __AssertUpdateBoundsFlatten<Pull<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Pull<T>, T> for filter::Value<T> {}
impl<T> __AssertUpdateBoundsFlatten<Pull<T>, T> for &filter::Value<T> {}
impl<T> __AssertUpdateBoundsFlatten<Pull<T>, T> for Pull<T> {}
impl<T> __AssertUpdateBoundsFlatten<Pull<T>, T> for &Pull<T> {}

impl<T> __AssertUpdateBounds<Pop<T>, T> for i32 {}
impl<T> __AssertUpdateBoundsFlatten<Pop<T>, T> for i32 {}
impl<T> __AssertUpdateBoundsFlatten<Pop<T>, T> for Pop<T> {}
impl<T> __AssertUpdateBoundsFlatten<Pop<T>, T> for &Pop<T> {}

impl<T> __AssertUpdateBounds<PullAll<T>, T> for T {}
impl<T> __AssertUpdateBounds<PullAll<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<PullAll<T>, T> for T {}
impl<T> __AssertUpdateBoundsFlatten<PullAll<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<PullAll<T>, T> for PullAll<T> {}
impl<T> __AssertUpdateBoundsFlatten<PullAll<T>, T> for &PullAll<T> {}

impl<T> __AssertUpdateBounds<Bit<T>, T> for T {}
impl<T> __AssertUpdateBounds<Bit<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Bit<T>, T> for T {}
impl<T> __AssertUpdateBoundsFlatten<Bit<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<Bit<T>, T> for Bit<T> {}
impl<T> __AssertUpdateBoundsFlatten<Bit<T>, T> for &Bit<T> {}

impl<T> __AssertUpdateBounds<SetOnInsert<T>, T> for T {}
impl<T> __AssertUpdateBounds<SetOnInsert<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<SetOnInsert<T>, T> for T {}
impl<T> __AssertUpdateBoundsFlatten<SetOnInsert<T>, T> for &T {}
impl<T> __AssertUpdateBoundsFlatten<SetOnInsert<T>, T> for SetOnInsert<T> {}
impl<T> __AssertUpdateBoundsFlatten<SetOnInsert<T>, T> for &SetOnInsert<T> {}

pub use macros::mongo_update_query as update;
