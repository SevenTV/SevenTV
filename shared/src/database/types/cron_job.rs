use super::MongoGenericCollection;
use crate::database::{Id, MongoCollection};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum CronJobId {
	EmoteScoresUpdate = 0,
	SubscriptionRefresh = 1,
}

impl From<CronJobId> for bson::Bson {
	fn from(value: CronJobId) -> Self {
		(value as i32).into()
	}
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "cron_jobs")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(enabled = 1, next_run = 1, held_until = 1)))]
#[serde(deny_unknown_fields)]
pub struct CronJob {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: CronJobId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	#[serde(with = "crate::database::serde")]
	pub last_run: Option<chrono::DateTime<chrono::Utc>>,
	#[serde(with = "crate::database::serde")]
	pub next_run: chrono::DateTime<chrono::Utc>,
	pub interval: CronJobInterval,
	pub enabled: bool,
	/// The machine id that is currently running the job
	pub currently_running_by: Option<Id<()>>,
	/// The time the job is free to be reclaimed by another machine
	#[serde(with = "crate::database::serde")]
	pub held_until: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(tag = "unit", content = "count")]
pub enum CronJobInterval {
	Hours(i32),
	Days(i32),
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<CronJob>()]
}

pub fn default_cron_jobs() -> Vec<CronJob> {
	vec![
		CronJob {
			id: CronJobId::EmoteScoresUpdate,
			name: "Emote Scores Update".to_string(),
			description: Some(
				"Automatically recalculates the emote scores so that search rankings are correct.".to_string(),
			),
			tags: vec!["emote".to_string()],
			last_run: None,
			next_run: chrono::Utc::now(),
			interval: CronJobInterval::Days(1),
			enabled: true,
			currently_running_by: None,
			held_until: chrono::Utc::now(),
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		},
		CronJob {
			id: CronJobId::SubscriptionRefresh,
			name: "Subscription Refresh".to_string(),
			description: Some(
				"Automatically refreshes subscriptions so that time based entitlements are given to the correct users."
					.to_string(),
			),
			tags: vec!["subscription".to_string()],
			last_run: None,
			next_run: chrono::Utc::now(),
			interval: CronJobInterval::Days(1),
			enabled: true,
			currently_running_by: None,
			held_until: chrono::Utc::now(),
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		},
	]
}
