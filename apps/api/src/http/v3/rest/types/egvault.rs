use shared::database::{
	paint::PaintId,
	product::{SubscriptionKind, SubscriptionProduct},
};

#[derive(Debug, serde::Serialize)]
pub struct Subscription {
	pub id: String,
	pub provider: Provider,
	/// Stripe product id
	pub product_id: String,
	/// Stripe price id
	pub plan: String,
	/// always 1
	pub seats: u32,
	/// Id of the user who is subscribed
	pub subscriber_id: String,
	/// Id of the user who is paying the subscription
	pub customer_id: String,
	pub started_at: chrono::DateTime<chrono::Utc>,
	pub ended_at: chrono::DateTime<chrono::Utc>,
	pub cycle: SubscriptionCycle,
	pub renew: bool,
	/// Date of the next renewal
	pub end_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
	Paypal,
	Stripe,
}

#[derive(Debug, serde::Serialize)]
pub struct SubscriptionCycle {
	pub timestamp: chrono::DateTime<chrono::Utc>,
	pub unit: SubscriptionCycleUnit,
	pub value: u32,
	pub status: SubscriptionCycleStatus,
	pub internal: bool,
	pub pending: bool,
	pub trial_end: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionCycleUnit {
	/// only one subscription has that
	Day,
	Month,
	Year,
}

#[derive(Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionCycleStatus {
	Ongoing,
	Ended,
	Canceled,
}

#[derive(Debug, serde::Serialize)]
pub struct Product {
	/// "subscription"
	pub name: String,
	pub plans: Vec<Plan>,
	pub current_paints: Vec<PaintId>,
}

#[derive(Debug, serde::Serialize)]
pub struct Plan {
	pub interval_unit: SubscriptionCycleUnit,
	/// always 1
	pub interval: u32,
	pub price: u64,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub discount: Option<f64>,
}

impl From<SubscriptionProduct> for Plan {
	fn from(value: SubscriptionProduct) -> Self {
		let (interval_unit, discount) = match value.kind {
			SubscriptionKind::Monthly => (SubscriptionCycleUnit::Month, None),
			SubscriptionKind::Yearly => (SubscriptionCycleUnit::Year, Some(0.2)),
		};

		Self {
			interval_unit,
			interval: 1,
			price: value
				.currency_prices
				.get(&value.default_currency)
				.copied()
				.unwrap_or_default()
				.max(0) as u64,
			discount,
		}
	}
}
