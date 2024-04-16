use std::collections::HashMap;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use crate::database::Collection;

mod permissions;

use bitmask_enum::bitmask;
use bson::oid::ObjectId;
use enum_impl::EnumImpl;

pub use self::permissions::*;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Role {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub badge_ids: Vec<ObjectId>,
	pub paint_ids: Vec<ObjectId>,
	pub emote_set_ids: Vec<ObjectId>,
	pub name: String,
	pub description: Option<String>,
	pub permissions: Permissions,
	pub hoist: bool,
	pub color: i32,
	pub tags: Vec<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Collection for Role {
	const NAME: &'static str = "roles";
}
