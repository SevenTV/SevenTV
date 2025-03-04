use std::collections::HashMap;

use super::duration::DurationUnit;
use super::{MongoCollection, MongoGenericCollection};
use crate::database::Id;
use crate::typesense::types::impl_typesense_type;

pub mod codes;
pub mod invoice;
pub mod special_event;
pub mod subscription;

/// A helper macro used to define newtypes for stripe IDs
macro_rules! stripe_type {
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
					Some(self.0.cmp(&other.0))
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

			impl crate::typesense::types::TypesenseType for $name {
				fn typesense_type() -> crate::typesense::types::FieldType {
					crate::typesense::types::FieldType::String
				}
			}

			async_graphql::scalar!($name);
		};
	};
}

stripe_type!(StripeProductId, stripe::PriceId);
stripe_type!(StripeSubscriptionId, stripe::SubscriptionId);
stripe_type!(InvoiceId, stripe::InvoiceId);
stripe_type!(InvoiceLineItemId, stripe::InvoiceLineItemId);
stripe_type!(CustomerId, stripe::CustomerId);
stripe_type!(PaymentIntentId, stripe::PaymentIntentId);

impl crate::typesense::types::TypesenseType for stripe::Currency {
	fn typesense_type() -> crate::typesense::types::FieldType {
		crate::typesense::types::FieldType::String
	}
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimePeriod {
	#[serde(with = "crate::database::serde")]
	pub start: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub end: chrono::DateTime<chrono::Utc>,
}

pub type ProductId = Id<Product>;

/// A non-recurring product, e.g. a paint bundle
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "products")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(search = "crate::typesense::types::product::Product")]
#[serde(deny_unknown_fields)]
pub struct Product {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: ProductId,
	pub provider_id: StripeProductId,
	pub active: bool,
	pub name: String,
	pub description: Option<String>,
	pub discount: Option<String>,
	pub extends_subscription: Option<SubscriptionProductId>,
	pub default_currency: stripe::Currency,
	pub currency_prices: HashMap<stripe::Currency, i64>,
	#[serde(with = "crate::database::serde")]
	pub created_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub type SubscriptionProductId = Id<SubscriptionProduct>;

/// There are only two kinds of subscriptions: monthly and yearly.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "subscription_products")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields("benifits.id" = 1)))]
#[mongo(search = "crate::typesense::types::product::SubscriptionProduct")]
#[serde(deny_unknown_fields)]
pub struct SubscriptionProduct {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SubscriptionProductId,
	pub provider_id: StripeProductId,
	pub variants: Vec<SubscriptionProductVariant>,
	pub default_variant_idx: i32,
	pub name: String,
	pub description: Option<String>,
	pub default_currency: stripe::Currency,
	pub benefits: Vec<SubscriptionBenefit>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionProductVariant {
	pub id: StripeProductId,
	pub paypal_id: Option<String>,
	pub active: bool,
	pub kind: SubscriptionProductKind,
	pub currency_prices: HashMap<stripe::Currency, i64>,
}

pub type SubscriptionBenefitId = Id<SubscriptionBenefit>;

// The `SubscriptionBenefitId` can have entitlements attached via the
// entitlement graph. If the user qualifies for the entitlement benefit then we
// create an edge between `Subscription` and `SubscriptionBenefit` on the
// entitlement graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionBenefit {
	pub id: SubscriptionBenefitId,
	pub name: String,
	pub condition: SubscriptionBenefitCondition,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum SubscriptionBenefitCondition {
	Duration(DurationUnit),
	TimePeriod(TimePeriod),
}

#[derive(Debug, Clone, PartialEq, Eq, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum SubscriptionProductKind {
	Monthly = 0,
	Yearly = 1,
}

impl_typesense_type!(SubscriptionProductKind, Int32);

impl SubscriptionProductKind {
	pub fn period_duration_months(&self) -> u32 {
		match self {
			SubscriptionProductKind::Monthly => 1,
			SubscriptionProductKind::Yearly => 12,
		}
	}
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[
		MongoGenericCollection::new::<Product>(),
		MongoGenericCollection::new::<SubscriptionProduct>(),
	]
	.into_iter()
	.chain(codes::mongo_collections())
	.chain(invoice::collections())
	.chain(subscription::collections())
	.chain(special_event::mongo_collections())
}
