use mongodb::bson::oid::ObjectId;
// use shared::old_types::BanEffect;

#[derive(Debug, serde::Deserialize)]
pub struct Ban {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub victim_id: ObjectId,
	pub actor_id: ObjectId,
	pub reason: String,
	pub expire_at: super::DateTime,
	// pub effects: BanEffect,
}
