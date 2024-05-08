use super::{PriceId, PurchaseId};
use crate::database::{Collection, Id, UserId};

// This is not intended to be used for discount/promo codes
// We use Stripe's coupon system for that

pub type RedeemCodeId = Id<RedeemCode>;

/// A redeem code is a code that can be redeemed for a product.
/// Redeeming a code is always free.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RedeemCode {
	#[serde(rename = "_id")]
	pub id: RedeemCodeId,
	pub name: String,
	pub code: String,
	pub description: Option<String>,
	pub enabled: bool,
	pub remaining_uses: Option<i32>,
	pub price_ids: Vec<PriceId>,
}

impl Collection for RedeemCode {
	const COLLECTION_NAME: &'static str = "redeem_codes";
}

pub type GiftCodeId = Id<GiftCode>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
/// A gift code involves a purchase that has already been made.
pub struct GiftCode {
	#[serde(rename = "_id")]
	pub id: GiftCodeId,
	pub owner_id: Option<UserId>,
	pub purchase_id: PurchaseId,
	pub name: String,
	pub code: String,
	pub description: Option<String>,
	pub enabled: bool,
}

impl Collection for GiftCode {
	const COLLECTION_NAME: &'static str = "gift_codes";
}
