use derive_builder::Builder;

use super::{ProductId, TimePeriod};
use crate::database::types::GenericCollection;
use crate::database::{Collection, Id};

pub type PromotionId = Id<Promotion>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct Promotion {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: PromotionId,
	pub name: String,
	#[builder(default)]
	pub description: Option<String>,
	#[builder(default)]
	pub tags: Vec<String>,
	pub products: Vec<PromotionProduct>,
	pub time_period: TimePeriod,
	pub unit_threshold: u32,
	/// If this promotion is publicly displayed to users
	pub public: bool,
	pub trigger: PromotionTrigger,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PromotionTrigger {
	Purchase,
	GiftRedeem,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Builder)]
pub struct PromotionProduct {
	pub id: ProductId,
	pub unit_value: u32,
}

impl Collection for Promotion {
	const COLLECTION_NAME: &'static str = "promotions";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"time_period.start": 1,
					"time_period.end": 1,
					"products.id": 1,
					"trigger": 1,
				})
				.build(),
		]
	}
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<Promotion>()]
}
