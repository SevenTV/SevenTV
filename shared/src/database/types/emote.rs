use bitmask_enum::bitmask;

use super::image_set::ImageSet;
use super::user::UserId;
use super::MongoGenericCollection;
use crate::database::{Id, MongoCollection};

pub type EmoteId = Id<Emote>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "emotes")]
#[mongo(index(fields(owner_id = 1)))]
#[mongo(index(fields("merged.target_id" = 1)))]
#[mongo(index(fields("merged.at" = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct Emote {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: EmoteId,
	pub owner_id: UserId,
	pub default_name: String,
	pub tags: Vec<String>,
	pub image_set: ImageSet,
	pub flags: EmoteFlags,
	pub aspect_ratio: f64,
	pub attribution: Vec<EmoteAttribution>,
	pub merged: Option<EmoteMerged>,
	pub scores: EmoteScores,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmoteScores {
	pub trending_day: i32,
	pub trending_week: i32,
	pub trending_month: i32,
	pub top_daily: i32,
	pub top_weekly: i32,
	pub top_monthly: i32,
	pub top_all_time: i32,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmoteMerged {
	pub target_id: EmoteId,
	#[serde(with = "crate::database::serde")]
	pub at: chrono::DateTime<chrono::Utc>,
}

#[bitmask(i32)]
pub enum EmoteFlags {
	PublicListed = 1 << 0,
	Private = 1 << 1,
	Nsfw = 1 << 2,
	DefaultZeroWidth = 1 << 3,
	ApprovedPersonal = 1 << 4,
	DeniedPersonal = 1 << 5,
	Animated = 1 << 6,
}

impl Default for EmoteFlags {
	fn default() -> Self {
		Self::none()
	}
}

impl From<EmoteFlags> for bson::Bson {
	fn from(value: EmoteFlags) -> Self {
		value.bits().into()
	}
}

impl serde::Serialize for EmoteFlags {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.bits().serialize(serializer)
	}
}

impl<'a> serde::Deserialize<'a> for EmoteFlags {
	fn deserialize<D>(deserializer: D) -> Result<EmoteFlags, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let bits = i32::deserialize(deserializer)?;
		Ok(EmoteFlags::from(bits))
	}
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmoteAttribution {
	pub user_id: UserId,
	#[serde(with = "crate::database::serde")]
	pub added_at: chrono::DateTime<chrono::Utc>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<Emote>()]
}
