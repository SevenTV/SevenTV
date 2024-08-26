use macros::MongoCollection;

use super::MongoGenericCollection;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "webhook_events")]
#[mongo(index(fields(_id = 1)))]
#[mongo(index(fields(created_at = 1), expire_after = 3600))]
#[serde(deny_unknown_fields)]
pub struct WebhookEvent {
    #[mongo(id)]
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(with = "crate::database::serde")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
    [MongoGenericCollection::new::<WebhookEvent>()]
}
