use std::time::Duration;

use derive_builder::Builder;

use super::connection::Platform;
use super::UserId;
use crate::database::types::GenericCollection;
use crate::database::{Collection, Id};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct UserPresence {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: Id<UserPresence>,
	pub platform: Platform,
	pub platform_room_id: String,
	pub user_id: UserId,
	pub authentic: bool,

	pub ip_address: std::net::IpAddr,
	#[builder(default)]
	#[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl Collection for UserPresence {
	const COLLECTION_NAME: &'static str = "user_presences";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"platform": 1,
					"platform_room_id": 1,
					"user_id": 1,
				})
				.options(mongodb::options::IndexOptions::builder().unique(true).build())
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"expires_at": 1,
				})
				.options(
					mongodb::options::IndexOptions::builder()
						.expire_after(Some(Duration::from_secs(0)))
						.build(),
				)
				.build(),
		]
	}
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<UserPresence>()]
}
