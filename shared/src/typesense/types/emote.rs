use chrono::Utc;

use super::{TypesenseCollection, TypesenseGenericCollection};
use crate::database;
use crate::database::emote::EmoteId;
use crate::database::user::UserId;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "emotes")]
#[serde(deny_unknown_fields)]
pub struct Emote {
	pub id: EmoteId,
	pub owner_id: UserId,
	pub default_name: String,
	pub tags: Vec<String>,
	pub flag_public_listed: bool,
	pub flag_private: bool,
	pub flag_nsfw: bool,
	pub flag_default_zero_width: bool,
	pub flag_approved_personal: bool,
	pub flag_denied_personal: bool,
	pub flag_animated: bool,
	pub aspect_ratio: f64,
	pub attribution: Vec<UserId>,
	pub merged_into: Option<EmoteId>,
	pub merged_at: Option<i64>,
	pub score_trending_day: i32,
	pub score_trending_week: i32,
	pub score_trending_month: i32,
	pub score_top_daily: i32,
	pub score_top_weekly: i32,
	pub score_top_monthly: i32,
	#[typesense(default_sort)]
	pub score_top_all_time: i32,
	pub deleted: bool,
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::emote::Emote> for Emote {
	fn from(value: database::emote::Emote) -> Self {
		Self {
			id: value.id,
			owner_id: value.owner_id,
			default_name: value.default_name,
			tags: value.tags,
			flag_public_listed: value.flags.contains(database::emote::EmoteFlags::PublicListed),
			flag_private: value.flags.contains(database::emote::EmoteFlags::Private),
			flag_nsfw: value.flags.contains(database::emote::EmoteFlags::Nsfw),
			flag_default_zero_width: value.flags.contains(database::emote::EmoteFlags::DefaultZeroWidth),
			flag_approved_personal: value.flags.contains(database::emote::EmoteFlags::ApprovedPersonal),
			flag_denied_personal: value.flags.contains(database::emote::EmoteFlags::DeniedPersonal),
			flag_animated: value.flags.contains(database::emote::EmoteFlags::Animated),
			aspect_ratio: value.aspect_ratio,
			attribution: value.attribution.into_iter().map(|a| a.user_id).collect(),
			merged_into: value.merged.as_ref().map(|m| m.target_id),
			merged_at: value.merged.as_ref().map(|m| m.at.timestamp_millis()),
			score_top_all_time: value.scores.top_all_time,
			score_top_daily: value.scores.top_daily,
			score_top_weekly: value.scores.top_weekly,
			score_top_monthly: value.scores.top_monthly,
			score_trending_day: value.scores.trending_day,
			score_trending_week: value.scores.trending_week,
			score_trending_month: value.scores.trending_month,
			deleted: value.deleted,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<Emote>()]
}
