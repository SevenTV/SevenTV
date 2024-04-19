use std::collections::HashMap;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use crate::database::{Collection, Id};

mod permissions;

use bitmask_enum::bitmask;
use enum_impl::EnumImpl;

pub use self::permissions::*;
use super::{BadgeId, EmoteSetId, PaintId};

pub type RoleId = Id<Role>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Role {
	#[serde(rename = "_id", skip_serializing_if = "Id::is_nil")]
	pub id: RoleId,
	pub badge_ids: Vec<BadgeId>,
	pub paint_ids: Vec<PaintId>,
	pub emote_set_ids: Vec<EmoteSetId>,
	pub name: String,
	pub description: Option<String>,
	pub permissions: Permissions,
	pub hoist: bool,
	pub color: i32,
	pub tags: Vec<String>,
}

impl Collection for Role {
	const COLLECTION_NAME: &'static str = "roles";
}
