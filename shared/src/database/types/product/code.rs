use super::{PriceId, PurchaseId};
use crate::database::{Collection, Id, UserId};

// This is not intended to be used for discount/promo codes
// We use Stripe's coupon system for that

pub type RedeemCodeId = Id<RedeemCode>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
/// A redeem code that can be used to redeem a product (acts as a 100% discount under the hood)
/// One potential use case is to make giveaway cards for specific products like paint bundles to give away at events
/// TODO: Does Stripe offer a native solition for that?
pub struct RedeemCode {
	#[serde(rename = "_id")]
	pub id: RedeemCodeId,
	pub name: String,
	pub description: Option<String>,
	pub enabled: bool,
	pub code: String,
	pub remaining_uses: Option<i32>,
	pub price_ids: Vec<PriceId>,
}

impl Collection for RedeemCode {
	const COLLECTION_NAME: &'static str = "redeem_codes";
}
