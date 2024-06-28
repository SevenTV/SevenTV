use std::collections::HashMap;

use super::duration::DurationUnit;
use super::{Collection, GenericCollection};

pub mod codes;
pub mod edge;
pub mod invoice;
pub mod promotion;
pub mod subscription;
pub mod subscription_timeline;

/// A helper macro used to define newtypes for stripe IDs
macro_rules! stripe_id {
	($name:ident, $inner:ty) => {
		pub struct $name(pub $inner);

		/// Implementations for the newtype
		const _: () = {
			impl std::fmt::Display for $name {
				fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
					write!(f, "{}", self.0)
				}
			}

			impl std::fmt::Debug for $name {
				fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
					write!(f, "{:?}", self.0)
				}
			}

			impl Clone for $name {
				fn clone(&self) -> Self {
					Self(self.0.clone())
				}
			}

			impl PartialEq for $name {
				fn eq(&self, other: &Self) -> bool {
					self.0 == other.0
				}
			}

			impl Eq for $name {}

			impl std::hash::Hash for $name {
				fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
					self.0.hash(state)
				}
			}

			impl PartialEq<$inner> for $name {
				fn eq(&self, other: &$inner) -> bool {
					self.0 == *other
				}
			}

			impl PartialEq<$name> for $inner {
				fn eq(&self, other: &$name) -> bool {
					*self == other.0
				}
			}

			impl PartialEq<&str> for $name {
				fn eq(&self, other: &&str) -> bool {
					self.0 == *other
				}
			}

			impl PartialEq<$name> for &str {
				fn eq(&self, other: &$name) -> bool {
					&other.0 == *self
				}
			}

			impl PartialOrd for $name {
				fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
					self.0.partial_cmp(&other.0)
				}
			}

			impl Ord for $name {
				fn cmp(&self, other: &Self) -> std::cmp::Ordering {
					self.0.cmp(&other.0)
				}
			}

			impl From<$inner> for $name {
				fn from(inner: $inner) -> Self {
					Self(inner)
				}
			}

			impl From<$name> for $inner {
				fn from(name: $name) -> Self {
					name.0
				}
			}

			impl std::str::FromStr for $name {
				type Err = <$inner as std::str::FromStr>::Err;

				fn from_str(s: &str) -> Result<Self, Self::Err> {
					Ok(Self(s.parse()?))
				}
			}

			impl std::ops::Deref for $name {
				type Target = $inner;

				fn deref(&self) -> &Self::Target {
					&self.0
				}
			}

			impl std::ops::DerefMut for $name {
				fn deref_mut(&mut self) -> &mut Self::Target {
					&mut self.0
				}
			}

			impl serde::Serialize for $name {
				fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
				where
					S: serde::Serializer,
				{
					self.0.serialize(serializer)
				}
			}

			impl<'de> serde::Deserialize<'de> for $name {
				fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
				where
					D: serde::Deserializer<'de>,
				{
					Ok(Self(<$inner as serde::Deserialize<'de>>::deserialize(deserializer)?))
				}
			}

			impl From<$name> for mongodb::bson::Bson {
				fn from(id: $name) -> Self {
					Self::String(id.to_string())
				}
			}
		};
	};
}

stripe_id!(ProductId, stripe::PriceId);
stripe_id!(SubscriptionId, stripe::SubscriptionId);
stripe_id!(InvoiceId, stripe::InvoiceId);
stripe_id!(InvoiceLineItemId, stripe::InvoiceLineItemId);
stripe_id!(CustomerId, stripe::CustomerId);

// An item that can be purchased
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
	#[serde(rename = "_id")]
	pub id: ProductId,
	pub name: String,
	pub description: Option<String>,
	pub recurring: Option<DurationUnit>,
	pub default_currency: stripe::Currency,
	pub currency_prices: HashMap<stripe::Currency, u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimePeriod {
	pub start: chrono::DateTime<chrono::Utc>,
	pub end: chrono::DateTime<chrono::Utc>,
}

impl Collection for Product {
	const COLLECTION_NAME: &'static str = "products";
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	std::iter::once(GenericCollection::new::<Product>())
		.chain(codes::collections())
		.chain(edge::collections())
		.chain(invoice::collections())
		.chain(promotion::collections())
		.chain(subscription::collections())
		.chain(subscription_timeline::collections())
}
