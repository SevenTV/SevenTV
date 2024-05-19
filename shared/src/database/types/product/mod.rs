use super::{BadgeId, Collection, EmoteSetId, PaintId, RoleId};
use crate::database::Id;

pub mod invoice;
pub mod purchase;

pub use invoice::*;
pub use purchase::*;

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

stripe_id!(ProductId, stripe::ProductId);
stripe_id!(ProductPriceId, stripe::PriceId);
stripe_id!(SubscriptionId, stripe::SubscriptionId);
stripe_id!(InvoiceId, stripe::InvoiceId);
stripe_id!(CustomerId, stripe::CustomerId);
stripe_id!(InvoiceLineItemId, stripe::InvoiceLineItemId);
stripe_id!(CouponId, stripe::CouponId);

// An item that can be purchased
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
	// This ID will be the stripe ID for the product
	#[serde(rename = "_id")]
	pub id: ProductId,
	pub kind: ProductKind,
	// there will be other fields here like name, description, price, etc.
	// those fields will be shown in the UI but are not relevant to the core logic
	// We should also make those fields sync from Stripe.
	pub prices: Vec<ProductPrice>,
	/// A list of entitlement groups that this product grants access to
	pub entitlement_group_ids: Vec<ProductEntitlementGroupId>,
	/// A group of products that conflict with this product
	/// In the case of one time purchases a user can only have one of the
	/// conflicting products In the case of subscriptions a user can only have
	/// one of the conflicting products active at a time This is useful for
	/// things like subscription tiers, or one time purchase upgrades, or
	/// bundles of one time purchases. The admin must ensure they configure the
	/// behavior of the conflicting products correctly.
	pub conflicting_product_ids: Vec<ProductId>,
	/// Remaing quantity of the product available for purchase, if None then the
	/// product has unlimited quantity. For subscriptions this is the max number
	/// of subscriptions that can be active at a time. For gift codes this limit
	/// is applied at the time of purchase.
	pub remaining: Option<u32>,
	/// This is only applicable to one time purchases, subscriptions will never
	/// have a max quantity as they are recurring For gift codes this limit is
	/// applied at the time of redemption, not at the time of purchase.
	pub max_user_quantity: Option<u32>,
}

pub type ProductBundleId = Id<ProductBundle>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductBundle {
	#[serde(rename = "_id")]
	pub id: ProductBundleId,
	/// The name of the bundle
	pub name: String,
	/// The description of the bundle
	pub description: String,
	/// The products that make up this bundle
	pub products: Vec<ProductRef>,
	/// The coupon that is applied to this bundle at checkout
	pub coupon: Option<CouponId>,
	/// The behavior of the bundle if the user already has some of the products
	/// in the bundle Product limits are only applied after this behavior is
	/// applied
	pub duplicate_behavior: ProductBundleDuplicateBehavior,
	/// If the bundle is giftable, when gifting a bundle to another user, this
	/// will generate multiple redeemable codes for each product in the bundle.
	/// The user can give these codes to other users to redeem the products in
	/// the bundle.
	pub giftable: bool,
	/// Remaining quantity of the bundle available for purchase, if None then
	/// the bundle has unlimited quantity. This limit is strictly for the
	/// bundle, however the products in the bundle may have their own limits. In
	/// the case of some of the products failing a limit the entire bundle will
	/// fail.
	pub remaining: Option<u32>,
	/// The maximum quantity of this bundle that a user can purchase
	/// This limit is strictly for the bundle, however the products in the
	/// bundle may have their own limits. In the case of some of the products
	/// failing a limit the entire bundle will fail. This does not take into
	/// account gifts as this is the limit for the user who is purchasing the
	/// bundle.
	pub max_user_quantity: Option<u32>,
	/// Expiry date of the bundle, after this date the bundle will no longer be
	/// available for purchase
	pub expiry: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum ProductBundleDuplicateBehavior {
	/// If the user already has some of the products in the bundle, allow them
	/// to purchase the rest using this bundle
	Allow = 0,
	/// If the user already has some of the products in the bundle, disallow
	/// them from purchasing the bundle
	Disallow = 1,
	/// If the user already has some of the products in the bundle, allow them
	/// to buy the bundle again with the same products
	Multiple = 2,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductPrice {
	pub id: ProductPriceId,
	/// The remaining quantity of the price available for purchase, if None then
	/// the price has unlimited quantity. This limit is applied to the price &
	/// the product, if the product has a limit then the price limit is the
	/// minimum of the two.
	pub remaining: Option<u32>,
	/// This is only applicable to one time purchases, subscriptions will never
	/// have a max quantity as they are recurring This limit is applied to the
	/// price & the product, if the product has a limit then the price limit is
	/// the minimum of the two.
	pub max_user_quantity: Option<u32>,
	// some other fields like currency, amount, etc.
}

// The kind of product
#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum ProductKind {
	Subscription = 0,
	OneTimePurchase = 1,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductRef {
	// The invoice id
	pub id: ProductId,
	// The item this reference refers to otherwise it is the whole invoice
	pub price_id: ProductPriceId,
}

impl Collection for Product {
	const COLLECTION_NAME: &'static str = "products";
}

pub type ProductEntitlementGroupId = Id<ProductEntitlementGroup>;

/// Entitlement groups are what manages the what access a product grants
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductEntitlementGroup {
	#[serde(rename = "_id")]
	pub id: ProductEntitlementGroupId,
	/// The name of the group
	pub name: String,
	/// The description of the group
	pub description: String,
	pub evaluations: Vec<ProductEntitlementGroupEvaluation>,
}

pub type ProductEntitlementGroupEvaluationId = Id<ProductEntitlementGroupEvaluation>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductEntitlementGroupEvaluation {
	/// An id of this evaluation
	pub id: ProductEntitlementGroupEvaluationId,
	/// The conditions that must be met for the entitlements to be granted
	pub conditions: Vec<ProductEntitlementGroupEvaluationCondition>,
	/// The entitlements that will be granted if the conditions are met
	pub entitlements: Vec<ProductEntitlement>,
	/// Specifies if the user must have an active product to receive these
	/// entitlements Typically this will always be true because you would not
	/// want to grant entitlements to users who have cancelled their
	/// subscription, or for one time purchases you would not want to grant
	/// entitlements to users who no longer have the product.
	pub requires_active: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", content = "data", rename_all = "snake_case")]
pub enum ProductEntitlementGroupEvaluationCondition {
	// Grant these entitlements if the user's has been subscribed for a certain period
	SubscriptionDuration(SubscriptionDuration),
	// Grant these entitlements if the user's subscription period renewed within a certain period, or
	// if the user's subscription period is active for the entire period (i.e, if the user has subscribed
	// for the entire period, but their renew interval is larger than the period)
	SubscriptionPeriod {
		/// The start of the period
		start: chrono::DateTime<chrono::Utc>,
		/// The end of the period
		end: chrono::DateTime<chrono::Utc>,
	},
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, tag = "scalar", content = "amount", rename_all = "snake_case")]
/// The duration of a subscription
/// Either in days or months, all other durations can be calculated from these
/// two, ie. a year is 12 months or a week is 7 days
pub enum SubscriptionDuration {
	/// Number of days
	Days(u32),
	/// Number of months
	Months(u32),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", content = "id", rename_all = "snake_case")]
pub enum ProductEntitlement {
	/// A role that would be granted to the user
	Role(RoleId),
	/// A badge that would be granted to the user
	Badge(BadgeId),
	/// A paint that would be granted to the user
	Paint(PaintId),
	/// An emote set that would be granted to the user
	EmoteSet(EmoteSetId),
	/// Products are special as they allow subscriptions to grant access to
	/// other products as if they had been purchased.
	Product(ProductId),
}
