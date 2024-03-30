use std::str::FromStr;

use crate::object_id::ObjectId;

pub fn parse_id(id: &str) -> Option<ulid::Ulid> {
	match id.len() {
		// ObjectIDs are 24 characters long
		24 => ObjectId::from_str(id).ok().map(|id| id.into()),
		// ULIDs are 26 characters long
		26 => ulid::Ulid::from_str(id).ok(),
		_ => None,
	}
}
