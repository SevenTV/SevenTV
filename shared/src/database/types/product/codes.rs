use serde::{Deserialize, Serialize};

use super::TimePeriod;
use crate::database::types::GenericCollection;
use crate::database::user::UserId;
use crate::database::{Collection, Id};

pub type GiftCodeId = Id<GiftCode>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GiftCode {
	#[serde(rename = "_id")]
	pub id: GiftCodeId,
	pub code: String,
	pub message: Option<String>,
	pub purchased_by: UserId,
	pub redeemed_by: Option<UserId>,
}

impl Collection for GiftCode {
	const COLLECTION_NAME: &'static str = "gift_codes";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"code": 1,
				})
				.options(mongodb::options::IndexOptions::builder().unique(true).build())
				.build(),
		]
	}
}

pub type RedeemCodeId = Id<RedeemCode>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemCode {
	pub id: RedeemCodeId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub code: String,
	pub remaining_uses: u32,
	pub active_period: TimePeriod,
}

impl Collection for RedeemCode {
	const COLLECTION_NAME: &'static str = "redeem_codes";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"code": 1,
				})
				.options(mongodb::options::IndexOptions::builder().unique(true).build())
				.build(),
		]
	}
}

pub type DiscountCodeId = Id<DiscountCode>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountCode {
	pub id: DiscountCodeId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub code: String,
	pub discount: Discount,
	pub active_period: TimePeriod,
}

impl Collection for DiscountCode {
	const COLLECTION_NAME: &'static str = "discount_codes";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"code": 1,
				})
				.options(mongodb::options::IndexOptions::builder().unique(true).build())
				.build(),
		]
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Discount {
	Percentage(f64),
	Amount(f64),
	Free,
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[
		GenericCollection::new::<GiftCode>(),
		GenericCollection::new::<RedeemCode>(),
		GenericCollection::new::<DiscountCode>(),
	]
}
