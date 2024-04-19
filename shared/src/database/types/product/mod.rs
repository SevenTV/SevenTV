mod code;
mod purchase;

pub use self::code::*;
pub use self::purchase::*;
use super::{BadgeId, Collection, EmoteSetId, PaintId, RoleId};
use crate::database::Id;

pub type ProductId = Id<Product>;

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
	#[serde(rename = "_id")]
	pub id: ProductId,
	pub name: String,
	pub description: String,
	pub tags: Vec<String>,
	pub enabled: bool,
	pub remaining_stock: Option<i32>,
	pub kind: ProductKind,
	pub rank: i16,
	pub price: f64,
	pub visibility: ProductVisibility,
	pub entitlement_groups: Vec<ProductEntitlementGroup>,
	pub giftable: ProductDataGiftable,
	pub subscription: Option<ProductDataSubscription>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum TimeInterval {
	/// Number of days
	/// This can be used to build weeks or years as well, being 7 or 365 days
	/// respectively.
	Day(u16),
	/// Number of months, this cannot be easily represented as a number of days
	/// because the number of days in a month is not constant
	/// When used in computing pricing 1 month is treated as 30 days. However
	/// when used to compute billing cycle. if they subscribe on the 1st of the
	/// month, they will be billed on the 1st of the month. If they subscribe on
	/// the 31st of the month they will be ideally billed on the 31st of each
	/// month unless, the month does not have 31 days, then it would be billed
	/// on the last day of the month.
	Month(u16),
}

/// A subscription product is special because the interval at which we bill
/// might not be
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductDataSubscription {
	pub base_interval: TimeInterval,
	/// If a subscription is pauseable, the user can pause their subscription
	/// and not be billed until they resume it. Meaning the subscription will be
	/// disabled and they will no longer have access to the subscription.
	/// This would work with credit'd time as well. If they buy a 1 year
	/// subscription and pause it for 6 months, they will have 6 months left
	/// when they resume. This option allows for the user to manage their sub,
	/// regardless of if the sub is pausable or not staff can always pause a
	/// sub.
	pub pauseable: bool,
	pub options: Vec<ProductDataSubscriptionOption>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductDataSubscriptionOption {
	/// The number of months for this interval
	pub interval: TimeInterval,
	/// If unset auto compute the price uses the base price * the product
	/// interval This allows for the ability to specify a discount for different
	/// intervals
	pub price: Option<VariantPrice>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum VariantPrice {
	/// Fixed price, ie. $5.00
	Fixed(f64),
	/// Percent of the original price, ie. 50% of $5.00 = $2.50, or 200% of
	/// $5.00 = $10.00
	Percent(f64),
	/// Fixed amount to add to the original price, ie. $5.00 + $2.50 = $7.50, or
	/// $5.00 - $2.50 = $2.50
	Amount(f64),
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum ProductDataGiftable {
	#[default]
	/// Can be gifted or bought for self
	Yes,
	/// Cannot be gifted, only bought for self
	No,
	/// Can only be gifted, not bought for self
	Required,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductEntitlementGroup {
	pub condition: Option<String>,
	pub entitlements: Vec<ProductEntitlement>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Copy)]
#[serde(tag = "kind", content = "id")]
#[serde(deny_unknown_fields)]
pub enum ProductEntitlement {
	Role(RoleId),
	Badge(BadgeId),
	Paint(PaintId),
	EmoteSet(EmoteSetId),
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum ProductKind {
	#[default]
	Base,
	Addon,
	Bundle,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum ProductVisibility {
	#[default]
	Public,
	Unlisted,
}

impl Collection for Product {
	const COLLECTION_NAME: &'static str = "products";
}
