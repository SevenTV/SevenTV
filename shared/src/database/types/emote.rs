use bitmask_enum::bitmask;

use super::image_set::ImageSet;
use super::user::UserId;
use super::GenericCollection;
use crate::database::{Collection, Id};

pub type EmoteId = Id<Emote>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Emote {
	#[serde(rename = "_id")]
	pub id: EmoteId,
	pub owner_id: UserId,
	pub default_name: String,
	pub tags: Vec<String>,
	pub animated: bool,
	pub image_set: ImageSet,
	pub flags: EmoteFlags,
	pub attribution: Vec<EmoteAttribution>,
	pub merged: Option<EmoteMerged>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmoteMerged {
	pub target_id: EmoteId,
	pub at: chrono::DateTime<chrono::Utc>,
}

impl Collection for Emote {
	const COLLECTION_NAME: &'static str = "emotes";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"owner_id": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"merged.target_id": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"merged.at": 1,
				})
				.build(),
		]
	}
}

#[bitmask(i32)]
pub enum EmoteFlags {
	PublicListed = 1 << 0,
	Private = 1 << 1,
	Nsfw = 1 << 2,
	DefaultZeroWidth = 1 << 3,
	ApprovedPersonal = 1 << 4,
	DeniedPersonal = 1 << 5,
}

impl Default for EmoteFlags {
	fn default() -> Self {
		Self::none()
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
	#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub added_at: chrono::DateTime<chrono::Utc>,
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<Emote>()]
}
