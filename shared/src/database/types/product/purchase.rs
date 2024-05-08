use crate::database::{Collection, Id};

pub type PurchaseId = Id<Purchase>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Purchase {
	#[serde(rename = "_id")]
	pub id: PurchaseId,
	pub was_gift: bool,
	pub data: PurchaseData,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "object")]
pub enum PurchaseData {
	Charge(stripe::Charge),
	Subscription(stripe::Subscription),
	SubscriptionSchedule(stripe::SubscriptionSchedule),
}

impl Collection for Purchase {
	const COLLECTION_NAME: &'static str = "purchases";
}
