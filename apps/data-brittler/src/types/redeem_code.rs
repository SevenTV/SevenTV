use super::EntitlementData;

#[derive(Debug, serde::Deserialize)]
pub struct RedeemCode {
	pub promotion: String,
	pub entitlements: Vec<RedeemEntitlement>,
}

#[derive(Debug, serde::Deserialize)]
pub struct RedeemEntitlement {
	pub app: RedeemApp,
	#[serde(flatten)]
	pub data: EntitlementData,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default)]
pub struct RedeemApp {
	pub name: String,
	pub event: String,
	pub state: Option<RedeemAppState>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default)]
pub struct RedeemAppState {
	pub trial_duration: Option<u32>,
}
