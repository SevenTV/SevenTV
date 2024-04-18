use mongodb::bson::oid::ObjectId;

pub fn object_id_from_datetime(dt: chrono::DateTime<chrono::Utc>) -> ObjectId {
	let timestamp = dt.timestamp() as u32;

	let mut bytes = ObjectId::new().bytes();
	bytes[..4].clone_from_slice(&timestamp.to_be_bytes());
	ObjectId::from_bytes(bytes)
}
