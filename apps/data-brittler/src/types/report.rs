use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Report {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub target_id: ObjectId,
	pub actor_id: ObjectId,
	pub status: ReportStatus,
	pub subject: String,
	pub body: String,
	pub assignee_ids: Vec<ObjectId>,
	pub last_updated_at: super::DateTime,
	pub closed_at: Option<super::DateTime>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReportStatus {
	Open,
	Closed,
}
