mod code;
mod purchase;

pub use self::code::*;
pub use self::purchase::*;
use super::{BadgeId, Collection, EmoteSetId, PaintId, RoleId};
use crate::database::Id;

pub type PriceId = Id<Price>;

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Price {
	#[serde(rename = "_id")]
	pub id: PriceId,
	pub tags: Vec<String>,
	pub rank: i16,
	pub entitlement_groups: Vec<PriceEntitlementGroup>,
	pub giftable: PriceDataGiftable,
	pub provider: GatewayProvider,
	pub provider_id: String,
	pub data: stripe::Price,
}

#[derive(Debug, Clone, Default, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum GatewayProvider {
	#[default]
	Stripe = 0,
	Paypal = 1,
}

#[derive(Debug, Clone, Default, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum PriceDataGiftable {
	/// Cannot be gifted, only bought for self
	No = 0,
	#[default]
	/// Can be gifted or bought for self
	Yes = 1,
	/// Can only be gifted, not bought for self
	Required = 2,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceEntitlementGroup {
	pub condition: Option<String>,
	pub entitlements: Vec<PriceEntitlement>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Copy)]
#[serde(tag = "kind", content = "id")]
#[serde(deny_unknown_fields)]
pub enum PriceEntitlement {
	Role(RoleId),
	Badge(BadgeId),
	Paint(PaintId),
	EmoteSet(EmoteSetId),
}

impl Collection for Price {
	const COLLECTION_NAME: &'static str = "prices";
}
