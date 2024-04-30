use mongodb::bson::oid::ObjectId;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct System {
    pub emote_set_id: ObjectId,
}
