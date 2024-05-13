use crate::database::{Collection, Id, UserId};

use super::{GatewayProvider, PriceId, RedeemCodeId};

pub type PurchaseId = Id<Purchase>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Purchase {
	#[serde(rename = "_id")]
	pub id: PurchaseId,
	pub purchase_kind: PurchaseKind,
	pub data: PurchaseData,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "kind", content = "data")]
pub enum PurchaseKind {
	Normal {
		user_id: UserId,
	},
	Gift {
		recipient: GiftRecipient,
		buyer_id: UserId,
		/// None if the gift has not been redeemed or accepted yet
		user_id: Option<UserId>,
	},
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "kind", content = "data")]
pub enum GiftRecipient {
	/// The gift can be redeemed by anyone
	Anyone {
		code: String,
	},
	/// The gift can only be redeemed by one of the specified users
	User {
		users: Vec<UserId>,
	},
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "kind", content = "data")]
pub enum PurchaseData {
	OneTime {
		items: Vec<PriceId>,
	},
	RedeemCode {
		id: RedeemCodeId,
	},
	Subscription {
		provider: GatewayProvider,
		/// old subscriptions are missing this field
		provider_id: Option<String>,
		status: SubscriptionStatus,
		prices: Vec<PriceId>,
		cancel_at_period_end: bool,
		cancel_at: Option<mongodb::bson::DateTime>,
		created: mongodb::bson::DateTime,
		ended_at: Option<mongodb::bson::DateTime>,
		trial_end: Option<mongodb::bson::DateTime>,
	},
	SubscriptionSchedule {
		provider_id: stripe::SubscriptionScheduleId,
		status: SubscriptionScheduleStatus,
		subscription_id: stripe::SubscriptionId,
		current_phase: SubscriptionScheduleCurrentPhase,
	},
}

#[derive(Copy, Clone, Debug, serde_repr::Deserialize_repr, serde_repr::Serialize_repr, Eq, PartialEq)]
#[repr(u8)]
pub enum SubscriptionStatus {
	Active = 0,
	Canceled = 1,
	Incomplete = 2,
	IncompleteExpired = 3,
	PastDue = 4,
	Paused = 5,
	Trialing = 6,
	Unpaid = 7,
}

impl From<stripe::SubscriptionStatus> for SubscriptionStatus {
	fn from(value: stripe::SubscriptionStatus) -> Self {
		match value {
			stripe::SubscriptionStatus::Active => Self::Active,
			stripe::SubscriptionStatus::Canceled => Self::Canceled,
			stripe::SubscriptionStatus::Incomplete => Self::Incomplete,
			stripe::SubscriptionStatus::IncompleteExpired => Self::IncompleteExpired,
			stripe::SubscriptionStatus::PastDue => Self::PastDue,
			stripe::SubscriptionStatus::Paused => Self::Paused,
			stripe::SubscriptionStatus::Trialing => Self::Trialing,
			stripe::SubscriptionStatus::Unpaid => Self::Unpaid,
		}
	}
}

impl From<SubscriptionStatus> for stripe::SubscriptionStatus {
	fn from(value: SubscriptionStatus) -> Self {
		match value {
			SubscriptionStatus::Active => Self::Active,
			SubscriptionStatus::Canceled => Self::Canceled,
			SubscriptionStatus::Incomplete => Self::Incomplete,
			SubscriptionStatus::IncompleteExpired => Self::IncompleteExpired,
			SubscriptionStatus::PastDue => Self::PastDue,
			SubscriptionStatus::Paused => Self::Paused,
			SubscriptionStatus::Trialing => Self::Trialing,
			SubscriptionStatus::Unpaid => Self::Unpaid,
		}
	}
}

#[derive(Copy, Clone, Debug, serde_repr::Deserialize_repr, serde_repr::Serialize_repr, Eq, PartialEq)]
#[repr(u8)]
pub enum SubscriptionScheduleStatus {
	Active = 0,
	Canceled = 1,
	Completed = 2,
	NotStarted = 3,
	Released = 4,
}

impl From<stripe::SubscriptionScheduleStatus> for SubscriptionScheduleStatus {
	fn from(value: stripe::SubscriptionScheduleStatus) -> Self {
		match value {
			stripe::SubscriptionScheduleStatus::Active => Self::Active,
			stripe::SubscriptionScheduleStatus::Canceled => Self::Canceled,
			stripe::SubscriptionScheduleStatus::Completed => Self::Completed,
			stripe::SubscriptionScheduleStatus::NotStarted => Self::NotStarted,
			stripe::SubscriptionScheduleStatus::Released => Self::Released,
		}
	}
}

impl From<SubscriptionScheduleStatus> for stripe::SubscriptionScheduleStatus {
	fn from(value: SubscriptionScheduleStatus) -> Self {
		match value {
			SubscriptionScheduleStatus::Active => Self::Active,
			SubscriptionScheduleStatus::Canceled => Self::Canceled,
			SubscriptionScheduleStatus::Completed => Self::Completed,
			SubscriptionScheduleStatus::NotStarted => Self::NotStarted,
			SubscriptionScheduleStatus::Released => Self::Released,
		}
	}
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionScheduleCurrentPhase {
	pub end_date: mongodb::bson::DateTime,
	pub start_date: mongodb::bson::DateTime,
}

impl From<stripe::SubscriptionScheduleCurrentPhase> for SubscriptionScheduleCurrentPhase {
	fn from(value: stripe::SubscriptionScheduleCurrentPhase) -> Self {
		Self {
			end_date: chrono::DateTime::from_timestamp(value.end_date, 0).unwrap().into(),
			start_date: chrono::DateTime::from_timestamp(value.start_date, 0).unwrap().into(),
		}
	}
}

impl From<SubscriptionScheduleCurrentPhase> for stripe::SubscriptionScheduleCurrentPhase {
	fn from(value: SubscriptionScheduleCurrentPhase) -> Self {
		Self {
			end_date: value.end_date.to_chrono().timestamp(),
			start_date: value.start_date.to_chrono().timestamp(),
		}
	}
}

impl Collection for Purchase {
	const COLLECTION_NAME: &'static str = "purchases";
}
