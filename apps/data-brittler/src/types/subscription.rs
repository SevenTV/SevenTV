use mongodb::bson::oid::ObjectId;
use shared::database::product::subscription::SubscriptionState;

#[derive(Debug, serde::Deserialize)]
pub struct Subscription {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub provider: SubscriptionProvider,
	#[serde(default, deserialize_with = "super::empty_string_is_none")]
	pub provider_id: Option<String>,
	pub started_at: super::DateTime,
	pub subscriber_id: ObjectId,
	pub customer_id: ObjectId,
	pub cycle: SubscriptionCycle,
	pub plan_id: String,
	pub product_id: String,
	/// always 1
	pub seats: i32,
	pub upgraded: Option<bool>,
	pub ended_at: Option<super::DateTime>,
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionProvider {
	Paypal,
	Stripe,
	#[serde(rename = "REDEEM_CODE")]
	RedeemCode,
	#[serde(rename = "nnys.live")]
	NnysLive,
	#[serde(rename = "")]
	None,
}

#[derive(Debug, serde::Deserialize)]
pub struct SubscriptionCycle {
	pub unit: SubscriptionCycleUnit,
	pub value: u32,
	pub status: SubscriptionCycleStatus,
	pub timestamp: Option<super::DateTime>,
	pub internal: bool,
	pub pending: bool,
	pub trial_end_at: Option<super::DateTime>,
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionCycleUnit {
	/// only one subscription has that
	Day,
	Month,
	Year,
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionCycleStatus {
	Ongoing,
	Ended,
	Canceled,
}

impl From<SubscriptionCycleStatus> for SubscriptionState {
	fn from(value: SubscriptionCycleStatus) -> Self {
		match value {
			SubscriptionCycleStatus::Ongoing => Self::Active,
			SubscriptionCycleStatus::Ended => Self::Ended,
			SubscriptionCycleStatus::Canceled => Self::CancelAtEnd,
		}
	}
}
