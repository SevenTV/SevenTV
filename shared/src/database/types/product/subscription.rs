use super::codes::{GiftCodeId, RedeemCodeId};
use super::{InvoiceId, InvoiceLineItemId, ProductId, SubscriptionId};
use crate::database::duration::DurationUnit;
use crate::database::types::MongoGenericCollection;
use crate::database::user::UserId;
use crate::database::{Id, MongoCollection};

pub type SubscriptionPeriodId = Id<SubscriptionPeriod>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, tag = "type")]
pub enum SubscriptionCreatedBy {
	RedeemCode {
		redeem_code_id: RedeemCodeId,
	},
	GiftCode {
		gift_code_id: GiftCodeId,
	},
	Invoice {
		invoice_id: InvoiceId,
		invoice_item_id: InvoiceLineItemId,
	},
	System {
		reason: Option<String>,
	},
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "subscription_periods")]
#[mongo(index(fields(user_id = 1)))]
#[mongo(index(fields(subscription_id = 1)))]
#[mongo(index(fields(start = 1)))]
#[mongo(index(fields(end = 1)))]
#[mongo(index(fields(product_ids = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct SubscriptionPeriod {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SubscriptionPeriodId,
	pub subscription_id: SubscriptionId,
	pub user_id: UserId,
	#[serde(with = "crate::database::serde")]
	pub start: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub end: chrono::DateTime<chrono::Utc>,
	pub is_trial: bool,
	pub created_by: SubscriptionCreatedBy,
	pub product_ids: Vec<ProductId>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub type SubscriptionCreditId = Id<SubscriptionCredit>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "subscription_credits")]
#[mongo(index(fields(user_id = 1)))]
#[mongo(index(fields(product_id = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct SubscriptionCredit {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SubscriptionCreditId,
	pub user_id: UserId,
	pub product_id: ProductId,
	pub duration: DurationUnit,
	pub created_by: SubscriptionCreatedBy,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[
		MongoGenericCollection::new::<SubscriptionPeriod>(),
		MongoGenericCollection::new::<SubscriptionCredit>(),
	]
}
