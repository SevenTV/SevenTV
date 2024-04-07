use shared::object_id::ObjectId;

#[derive(Debug, serde::Deserialize)]
pub struct Entitlement {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: Option<ObjectId>,
    #[serde(flatten)]
    pub data: EntitlementData,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntitlementData {
    Role {
        #[serde(rename = "ref")]
        ref_id: ObjectId,    
    },
    Paint {
        #[serde(rename = "ref")]
        ref_id: ObjectId,
        #[serde(default)]
        selected: bool,
    },
    Badge {
        #[serde(rename = "ref")]
        ref_id: ObjectId,
        #[serde(default)]
        selected: bool,
    },
    Subscription {},
    EmoteSet {},
}
