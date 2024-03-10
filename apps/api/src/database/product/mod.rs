use postgres_types::{FromSql, ToSql};

mod association_badge;
mod association_emote_set;
mod association_paint;
mod association_product;
mod association_role;
mod code;
mod code_association_product;
mod purchase;

pub use association_badge::*;
pub use association_emote_set::*;
pub use association_paint::*;
pub use association_product::*;
pub use association_role::*;
pub use code::*;
pub use code_association_product::*;
pub use purchase::*;

#[derive(Debug, Clone, postgres_from_row::FromRow)]
pub struct Product {
	pub id: ulid::Ulid,
	pub name: String,
	pub description: String,
	pub tags: Vec<String>,
	pub enabled: bool,
	pub remaining_stock: Option<i32>,
	pub kind: ProductKind,
	pub rank: i16,
	pub visibility: ProductVisibility,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: ProductData,
	pub updated_at: chrono::DateTime<chrono::Utc>,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProductData {
	#[serde(default)]
	pub entitlement_groups: Vec<ProductEntitlementGroup>,
	#[serde(default)]
	pub giftable: ProductDataGiftable,
	#[serde(default)]
	pub subscription: Option<ProductDataSubscription>,
	pub price: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// A subscription product is special because the interval at which we bill
/// might not be
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
	#[serde(default)]
	pub options: Vec<ProductDataSubscriptionOption>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProductDataSubscriptionOption {
	/// The number of months for this interval
	pub interval: TimeInterval,
	/// If unset auto compute the price uses the base price * the product
	/// interval This allows for the ability to specify a discount for different
	/// intervals
	#[serde(default)]
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
#[serde(default)]
pub struct ProductEntitlementGroup {
	pub condition: Option<String>,
	pub entitlements: Vec<ProductEntitlement>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "id")]
pub enum ProductEntitlement {
	Role(ulid::Ulid),
	Badge(ulid::Ulid),
	Paint(ulid::Ulid),
	EmoteSet(ulid::Ulid),
}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "product_kind")]
pub enum ProductKind {
	#[default]
	#[postgres(name = "BASE")]
	Base,
	#[postgres(name = "ADDON")]
	Addon,
	#[postgres(name = "BUNDLE")]
	Bundle,
}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "product_visibility")]
pub enum ProductVisibility {
	#[default]
	#[postgres(name = "PUBLIC")]
	Public,
	#[postgres(name = "UNLISTED")]
	Unlisted,
}
