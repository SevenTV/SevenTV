use shared::database::paint::PaintId;
use shared::database::product::subscription::SubscriptionState;
use shared::database::product::{SubscriptionProductKind, SubscriptionProductVariant};

#[derive(Debug, serde::Serialize)]
pub struct Subscription {
	pub id: String,
	pub provider: Option<Provider>,
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
	pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
	pub cycle: SubscriptionCycle,
	pub renew: bool,
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
	pub unit: Option<SubscriptionCycleUnit>,
	pub value: u32,
	pub status: SubscriptionCycleStatus,
	pub internal: bool,
	pub pending: bool,
	pub trial_end: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionCycleUnit {
	/// only one subscription has that
	Day,
	Month,
	Year,
}

impl From<SubscriptionProductKind> for SubscriptionCycleUnit {
	fn from(value: SubscriptionProductKind) -> Self {
		match value {
			SubscriptionProductKind::Monthly => Self::Month,
			SubscriptionProductKind::Yearly => Self::Year,
		}
	}
}

#[derive(Debug, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionCycleStatus {
	Ongoing,
	Ended,
	Canceled,
}

impl From<SubscriptionState> for SubscriptionCycleStatus {
	fn from(value: SubscriptionState) -> Self {
		match value {
			SubscriptionState::Active => Self::Ongoing,
			SubscriptionState::Ended => Self::Ended,
			SubscriptionState::CancelAtEnd => Self::Canceled,
		}
	}
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
	pub currency: stripe::Currency,
}

impl Plan {
	pub fn from_variant(
		value: SubscriptionProductVariant,
		regional_currency: Option<stripe::Currency>,
		default_currency: stripe::Currency,
	) -> Option<Self> {
		let (interval_unit, discount) = match value.kind {
			SubscriptionProductKind::Monthly => (SubscriptionCycleUnit::Month, None),
			SubscriptionProductKind::Yearly => (SubscriptionCycleUnit::Year, Some(0.2)),
		};

		let (currency, price) =
			if let Some(price) = regional_currency.and_then(|currency| value.currency_prices.get(&currency)) {
				(regional_currency.unwrap(), *price)
			} else if let Some(price) = value.currency_prices.get(&default_currency) {
				(default_currency, *price)
			} else {
				return None;
			};

		Some(Self {
			interval_unit,
			interval: 1,
			price: price.max(0) as u64,
			currency,
			discount,
		})
	}
}
