use super::{ProductId, TimePeriod};
use crate::database::types::MongoGenericCollection;
use crate::database::user::UserId;
use crate::database::{Id, MongoCollection};
use crate::typesense::types::impl_typesense_type;

pub type PromotionId = Id<Promotion>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "promotions")]
#[mongo(index(fields("time_period.start" = 1, "time_period.end" = 1, "products.id" = 1, trigger = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct Promotion {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: PromotionId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub products: Vec<PromotionProduct>,
	pub time_period: TimePeriod,
	pub unit_threshold: i32,
	/// If this promotion is publicly displayed to users
	pub public: bool,
	pub created_by: UserId,
	pub trigger: PromotionTrigger,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum PromotionTrigger {
	Purchase = 0,
	GiftRedeem = 1,
}

impl_typesense_type!(PromotionTrigger, Int32);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromotionProduct {
	pub id: ProductId,
	pub unit_value: i32,
}

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<Promotion>()]
}
