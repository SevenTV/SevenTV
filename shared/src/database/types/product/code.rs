use super::{PriceId, PurchaseId};
use crate::database::{Collection, Id, UserId};

// This is not intended to be used for discount/promo codes
// We use Stripe's coupon system for that

pub type RedeemCodeId = Id<RedeemCode>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RedeemCode {
	#[serde(rename = "_id")]
	pub id: RedeemCodeId,
	pub name: String,
	pub description: Option<String>,
	pub enabled: bool,
	pub recipient: RedeemCodeRecipient,
	pub redeem_type: RedeemCodeType,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "data")]
pub enum RedeemCodeRecipient {
	/// A code that can be redeemed by anyone.
	Anyone {
		code: String,
		remaining_uses: Option<i32>,
	},
	/// A code that can only be redeemed by a specific user.
	User {
		user_id: UserId,
	},
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "data")]
pub enum RedeemCodeType {
	/// A code that grants access to the listed prices out of nothing (ex nihilo).
	ExNihilo {
		price_ids: Vec<PriceId>,
	},
	/// A gift code with an associated purchase that has already been made.
	Gift {
		purchase_id: PurchaseId,
	},
}

impl Collection for RedeemCode {
	const COLLECTION_NAME: &'static str = "redeem_codes";
}
