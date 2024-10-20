use std::collections::HashMap;

use macros::MongoCollection;

use super::MongoGenericCollection;
use crate::database::Id;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "paypal_webhook_events")]
#[mongo(index(fields(_id = 1)))]
#[serde(deny_unknown_fields)]
pub struct PaypalWebhookEvent {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: Id<PaypalWebhookEvent>,
	pub headers: HashMap<String, String>,
	pub event: serde_json::Value,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<PaypalWebhookEvent>()]
}
