use bitmask_enum::bitmask;
use mongodb::bson::oid::ObjectId;

#[derive(Debug, serde::Deserialize)]
pub struct Ban {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub victim_id: ObjectId,
	pub actor_id: ObjectId,
	pub reason: String,
	pub expire_at: super::DateTime,
	pub effects: BanEffect,
}

#[bitmask(i64)]
// https://github.com/SevenTV/Common/blob/master/structures/v3/type.ban.go#L29
pub enum BanEffect {
	NoPermissions = 1 << 0,
	NoAuth = 1 << 1,
	NoOwnership = 1 << 2,
	MemoryHole = 1 << 3,
	BlockedIp = 1 << 4,
}

impl Default for BanEffect {
	fn default() -> Self {
		BanEffect::none()
	}
}

impl<'a> serde::Deserialize<'a> for BanEffect {
	fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<BanEffect, D::Error> {
		let bits = i64::deserialize(deserializer)?;
		Ok(BanEffect::from(bits))
	}
}
