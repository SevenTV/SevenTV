use derive_builder::Builder;

use super::{ProductId, SubscriptionId};
use crate::database::types::GenericCollection;
use crate::database::user::UserId;
use crate::database::{Collection, Id};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct Subscription {
	#[serde(rename = "_id")]
	pub id: SubscriptionId,
	pub user_id: UserId,
	pub periods: Vec<SubscriptionPeriod>,
}

pub type SubscriptionPeriodId = Id<SubscriptionPeriod>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionPeriod {
	#[builder(default)]
	pub id: SubscriptionPeriodId,
	pub start: chrono::DateTime<chrono::Utc>,
	pub end: chrono::DateTime<chrono::Utc>,
	pub product_ids: Vec<ProductId>,
}

impl Collection for Subscription {
	const COLLECTION_NAME: &'static str = "subscriptions";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"user_id": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"periods.id": 1,
				})
				.options(mongodb::options::IndexOptions::builder().unique(true).build())
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"periods.product_ids": 1,
				})
				.build(),
		]
	}
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<Subscription>()]
}
