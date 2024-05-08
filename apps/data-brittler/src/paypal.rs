use std::str::FromStr;

use serde::Deserialize;

use crate::config::PaypalConfig;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Plan {
	pub id: String,
	pub name: String,
	pub status: PlanStatus,
	pub description: Option<String>,
	pub billing_cycles: Vec<BillingCycle>,
	pub create_time: chrono::DateTime<chrono::Utc>,
	pub update_time: chrono::DateTime<chrono::Utc>,
}

impl Plan {
	pub async fn query_by_id(id: &str, config: &PaypalConfig) -> reqwest::Result<Self> {
		let client = reqwest::Client::new();
		client
			.get(format!("https://api.paypal.com/v1/billing/plans/{}", id))
			.basic_auth(&config.client_id, Some(&config.client_secret))
			.send()
			.await?
			.error_for_status()?
			.json()
			.await
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlanStatus {
	Created,
	Inactive,
	Active,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BillingCycle {
	pub tenure_type: BillingCycleTenureType,
	pub sequence: u32,
	pub pricing_scheme: BillingCyclePricingScheme,
	pub frequency: BillingCycleFrequency,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BillingCycleTenureType {
	Regular,
	Trial,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BillingCyclePricingScheme {
	pub fixed_price: FixedPrice,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FixedPrice {
	pub value: String,
	#[serde(deserialize_with = "currency_code_deserialize")]
	pub currency_code: stripe::Currency,
}

fn currency_code_deserialize<'d, D: serde::Deserializer<'d>>(d: D) -> Result<stripe::Currency, D::Error> {
	let s = String::deserialize(d)?;
	stripe::Currency::from_str(&s.to_lowercase()).map_err(serde::de::Error::custom)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BillingCycleFrequency {
	pub interval_unit: BillingCycleFrequencyIntervalUnit,
	pub interval_count: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BillingCycleFrequencyIntervalUnit {
	Day,
	Week,
	Month,
	Year,
}

impl From<BillingCycleFrequencyIntervalUnit> for stripe::RecurringInterval {
	fn from(value: BillingCycleFrequencyIntervalUnit) -> Self {
		match value {
			BillingCycleFrequencyIntervalUnit::Day => Self::Day,
			BillingCycleFrequencyIntervalUnit::Week => Self::Week,
			BillingCycleFrequencyIntervalUnit::Month => Self::Month,
			BillingCycleFrequencyIntervalUnit::Year => Self::Year,
		}
	}
}
