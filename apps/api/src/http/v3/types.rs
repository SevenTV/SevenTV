use shared::object_id::ObjectId;
use shared::types::{ImageHost, UserModelPartial};

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Emote {
	id: ObjectId,
	name: String,
	flags: i32,
	tags: Vec<String>,
	lifecycle: i32,
	state: Vec<EmoteVersionState>,
	listed: bool,
	animated: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	owner: Option<UserModelPartial>,
	host: ImageHost,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	versions: Vec<EmoteVersion>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct EmoteVersion {
	id: ObjectId,
	name: String,
	description: String,
	lifecycle: i32,
	state: Vec<EmoteVersionState>,
	listed: bool,
	animated: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	host: Option<ImageHost>,
	created_at: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmoteVersionState {
	Listed,
	Personal,
	NoPersonal,
}
