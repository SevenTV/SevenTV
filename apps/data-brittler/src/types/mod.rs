use serde::{Deserialize, Deserializer};

mod audit_log;
mod ban;
mod cosmetic;
mod emote;
mod emote_set;
mod entitlement;
mod image_file;
mod message;
mod price;
mod report;
mod role;
mod subscription;
mod system;
mod user;

pub use audit_log::*;
pub use ban::*;
pub use cosmetic::*;
pub use emote::*;
pub use emote_set::*;
pub use entitlement::*;
pub use image_file::*;
pub use message::*;
pub use price::*;
pub use report::*;
pub use role::*;
pub use subscription::*;
pub use system::*;
pub use user::*;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum DateTime {
	Bson(mongodb::bson::DateTime),
	Chrono(chrono::DateTime<chrono::Utc>),
}

impl Default for DateTime {
	fn default() -> Self {
		DateTime::Bson(mongodb::bson::DateTime::MIN)
	}
}

impl DateTime {
	pub fn into_chrono(self) -> chrono::DateTime<chrono::Utc> {
		self.into()
	}
}

impl From<DateTime> for chrono::DateTime<chrono::Utc> {
	fn from(value: DateTime) -> Self {
		match value {
			DateTime::Bson(d) => d.to_chrono(),
			DateTime::Chrono(d) => d,
		}
	}
}

impl From<DateTime> for mongodb::bson::DateTime {
	fn from(value: DateTime) -> Self {
		match value {
			DateTime::Bson(d) => d,
			DateTime::Chrono(d) => d.into(),
		}
	}
}

fn empty_string_is_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
	D: Deserializer<'de>,
{
	let s = String::deserialize(deserializer)?;
	if s.is_empty() {
		Ok(None)
	} else {
		Ok(Some(s))
	}
}

fn unsigned_int<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
	D: Deserializer<'de>,
{
	let i = i32::deserialize(deserializer)?;
	Ok(i.max(0) as u32)
}

fn null_to_default<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
	T: Default + Deserialize<'de>,
	D: Deserializer<'de>,
{
	let opt = Option::deserialize(deserializer)?;
	Ok(opt.unwrap_or_default())
}
