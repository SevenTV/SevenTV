use crate::database::Table;

mod badge;
mod emote_set;
mod paint;

use anyhow::Context;
use serde::de;
use serde::ser::SerializeMap;
use serde_json::json;

pub use self::badge::*;
pub use self::emote_set::*;
pub use self::paint::*;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct Role {
	pub id: ulid::Ulid,
	pub name: String,
	pub description: Option<String>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: RoleData,
	pub priority: i16,
	pub hoist: bool,
	pub color: i32,
	pub tags: Vec<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub trait PermissionsExt {
	fn has_permission(&self, permission: &RolePermission) -> Option<bool>;

	fn has_any_permission<'a>(&self, permissions: impl IntoIterator<Item = &'a RolePermission>) -> Option<bool> {
		let mut has_permission = false;

		for permission in permissions {
			match self.has_permission(permission) {
				Some(true) => return Some(true),
				Some(false) => has_permission = true,
				None => {}
			}
		}

		if has_permission { Some(false) } else { None }
	}

	fn has_all_permissions<'a>(&self, permissions: impl IntoIterator<Item = &'a RolePermission>) -> Option<bool> {
		for permission in permissions {
			if !self.has_permission(permission)? {
				return Some(false);
			}
		}

		Some(true)
	}
}

impl PermissionsExt for &[RolePermission] {
	/// Returns true if any of the permissions in the slice match the given
	/// permission
	fn has_permission(&self, permission: &RolePermission) -> Option<bool> {
		let mut has_permission = false;

		for p in *self {
			match p.has_permission(permission) {
				Some(true) => return Some(true),
				Some(false) => has_permission = true,
				None => {}
			}
		}

		if has_permission { Some(false) } else { None }
	}
}

impl PermissionsExt for Vec<RolePermission> {
	fn has_permission(&self, permission: &RolePermission) -> Option<bool> {
		(&self as &[RolePermission]).has_permission(permission)
	}
}

impl PermissionsExt for Role {
	fn has_permission(&self, permission: &RolePermission) -> Option<bool> {
		self.data.has_permission(permission)
	}
}

impl Table for Role {
	const TABLE_NAME: &'static str = "roles";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct RoleData {
	// TODO: permissions
	#[serde(deserialize_with = "deserialize_permissions")]
	#[serde(serialize_with = "serialize_permissions")]
	/// In the database this is a Map of kind => value where kind is a string
	/// and value is a JSON value. Meaning that duplicate keys are not allowed
	/// and the keys are not ordered.
	pub permissions: Vec<RolePermission>,
}

impl PermissionsExt for RoleData {
	fn has_permission(&self, permission: &RolePermission) -> Option<bool> {
		self.permissions.has_permission(permission)
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(tag = "kind", content = "value")]
pub enum RolePermission {
	/// EmoteUpload means uploading a new emote
	EmoteUpload(bool),
	/// EmoteDelete means deleting an emote you own
	EmoteDelete(bool),
	/// EmoteEdit means changing the name, description, and tags of an emote you
	/// own
	EmoteEdit(bool),
	/// EmoteAmin means changing any emote's name, description, and tags
	EmoteAdmin(bool),

	/// RoleAdmin means creating, editing, and deleting roles
	RoleAdmin(bool),
	/// RoleAssign means assigning roles to users
	RoleAssign(bool),

	/// EmoteSetCreate means creating a new emote set
	EmoteSetCreate(bool),
	/// EmoteSetDelete means deleting an emote set you own
	EmoteSetDelete(bool),
	/// EmoteSetAdmin means changing the name and description of any emote set
	EmoteSetAdmin(bool),

	/// BadgeAdmin means creating, editing, and deleting badges
	BadgeAdmin(bool),

	/// PaintAdmin means creating, editing, and deleting paints
	PaintAdmin(bool),

	/// UserBan means banning or unbanning users
	UserBan(bool),
	/// UserMerge means merging users together
	UserMerge(bool),
	/// UserDelete means deleting users
	UserDelete(bool),
	/// UserEdit means changing user data
	UserEdit(bool),
	/// UserAdmin means changing any user's data or perform any action on any
	/// user
	UserAdmin(bool),

	/// FeatureAnimatedProfilePicture allows a user to use an animated profile
	/// picture
	FeatureAnimatedProfilePicture(bool),
	/// FeaturePersonalEmotes allows a user to create/use a personal emote set
	FeaturePersonalEmoteSet(bool),
	/// FeatureBadge allows a user to use a badge (if they have one)
	FeatureBadge(bool),
	/// FeatureEmoteSetCountLimit sets the number of emote sets a user has
	FeatureEmoteSetCountLimit(u16),
	/// FeatureEmoteSlotsLimit sets the number of emote slots a user has
	FeatureEmoteSetSlotsLimit(u16),
	/// FeaturePersonalEmoteSetCountLimit sets the number of emotes in a
	/// personal emote set
	FeaturePersonalEmoteSlotsLimit(u16),

	/// Admin grants all permissions to the user, and limits are ignored however
	/// role orders are still respected.
	Admin(bool),

	/// SuperAdmin grants all permissions to the user, limits and role orders
	/// are ignored.
	SuperAdmin(bool),

	/// This value is not a recognized permission
	Unknown(String, serde_json::Value),
}

impl PartialEq for RolePermission {
	fn eq(&self, other: &Self) -> bool {
		self.partial_cmp(other) == Some(std::cmp::Ordering::Equal)
	}
}

impl PartialOrd for RolePermission {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		fn cmp_bool(a: &bool, b: &bool) -> std::cmp::Ordering {
			if a == b {
				std::cmp::Ordering::Equal
			} else {
				std::cmp::Ordering::Less
			}
		}

		match (self, other) {
			(Self::EmoteUpload(v1), Self::EmoteUpload(v2)) => Some(cmp_bool(v1, v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::EmoteUpload(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::EmoteUpload(_)) => None,

			(Self::EmoteDelete(v1), Self::EmoteDelete(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::EmoteDelete(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::EmoteDelete(_)) => None,

			(Self::EmoteEdit(v1), Self::EmoteEdit(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::EmoteEdit(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::EmoteEdit(_)) => None,

			(Self::EmoteAdmin(v1), Self::EmoteAdmin(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::EmoteAdmin(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::EmoteAdmin(_)) => None,

			(Self::RoleAdmin(v1), Self::RoleAdmin(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::RoleAdmin(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::RoleAdmin(_)) => None,

			(Self::RoleAssign(v1), Self::RoleAssign(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::RoleAssign(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::RoleAssign(_)) => None,

			(Self::EmoteSetCreate(v1), Self::EmoteSetCreate(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::EmoteSetCreate(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::EmoteSetCreate(_)) => None,

			(Self::EmoteSetDelete(v1), Self::EmoteSetDelete(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::EmoteSetDelete(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::EmoteSetDelete(_)) => None,

			(Self::EmoteSetAdmin(v1), Self::EmoteSetAdmin(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::EmoteSetAdmin(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::EmoteSetAdmin(_)) => None,

			(Self::BadgeAdmin(v1), Self::BadgeAdmin(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::BadgeAdmin(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::BadgeAdmin(_)) => None,

			(Self::PaintAdmin(v1), Self::PaintAdmin(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::PaintAdmin(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::PaintAdmin(_)) => None,

			(Self::UserBan(v1), Self::UserBan(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::UserBan(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::UserBan(_)) => None,

			(Self::UserMerge(v1), Self::UserMerge(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::UserMerge(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::UserMerge(_)) => None,

			(Self::UserDelete(v1), Self::UserDelete(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::UserDelete(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::UserDelete(_)) => None,

			(Self::UserEdit(v1), Self::UserEdit(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::UserEdit(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::UserEdit(_)) => None,

			(Self::UserAdmin(v1), Self::UserAdmin(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::UserAdmin(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::UserAdmin(_)) => None,

			(Self::FeatureAnimatedProfilePicture(v1), Self::FeatureAnimatedProfilePicture(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::FeatureAnimatedProfilePicture(v2)) => {
				Some(cmp_bool(&true, &v2))
			}
			(_, Self::FeatureAnimatedProfilePicture(_)) => None,

			(Self::FeaturePersonalEmoteSet(v1), Self::FeaturePersonalEmoteSet(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::FeaturePersonalEmoteSet(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::FeaturePersonalEmoteSet(_)) => None,

			(Self::FeatureBadge(v1), Self::FeatureBadge(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::FeatureBadge(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::FeatureBadge(_)) => None,

			(Self::FeatureEmoteSetCountLimit(v1), Self::FeatureEmoteSetCountLimit(v2)) => Some(v1.cmp(&v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::FeatureEmoteSetCountLimit(_)) => {
				Some(std::cmp::Ordering::Greater)
			}
			(_, Self::FeatureEmoteSetCountLimit(_)) => None,

			(Self::FeatureEmoteSetSlotsLimit(v1), Self::FeatureEmoteSetSlotsLimit(v2)) => Some(v1.cmp(&v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::FeatureEmoteSetSlotsLimit(_)) => {
				Some(std::cmp::Ordering::Greater)
			}
			(_, Self::FeatureEmoteSetSlotsLimit(_)) => None,

			(Self::FeaturePersonalEmoteSlotsLimit(v1), Self::FeaturePersonalEmoteSlotsLimit(v2)) => Some(v1.cmp(&v2)),
			(Self::SuperAdmin(true) | Self::Admin(true), Self::FeaturePersonalEmoteSlotsLimit(_)) => {
				Some(std::cmp::Ordering::Greater)
			}
			(_, Self::FeaturePersonalEmoteSlotsLimit(_)) => None,

			(Self::Admin(v1), Self::Admin(v2)) => Some(cmp_bool(&v1, &v2)),
			(Self::SuperAdmin(true), Self::Admin(v2)) => Some(cmp_bool(&true, &v2)),
			(_, Self::Admin(_)) => None,

			(Self::SuperAdmin(v1), Self::SuperAdmin(v2)) => Some(cmp_bool(&v1, &v2)),
			(_, Self::SuperAdmin(_)) => None,

			(Self::Unknown(_, _), _) => None,
			(_, Self::Unknown(_, _)) => None,
		}
	}
}

impl RolePermission {
	fn from_parts(kind: String, value: serde_json::Value) -> Self {
		let map = json!({
			"kind": kind,
			"value": value,
		});

		serde_json::from_value(map).unwrap_or_else(|_| Self::Unknown(kind, value))
	}

	fn into_parts(&self) -> anyhow::Result<(String, serde_json::Value)> {
		match self {
			Self::Unknown(kind, value) => Ok((kind.clone(), value.clone())),
			permission => {
				let mut value = serde_json::to_value(permission).context("failed to serialize permission")?;
				let map = value.as_object_mut().context("permission value is not an object")?;
				let kind = match map.remove("kind").context("permission kind is missing")? {
					serde_json::Value::String(kind) => kind,
					_ => anyhow::bail!("permission kind is not a string"),
				};
				let value = map.remove("value").context("permission value is missing")?;

				Ok((kind, value))
			}
		}
	}
}

impl PermissionsExt for RolePermission {
	fn has_permission(&self, permission: &RolePermission) -> Option<bool> {
		Some(matches!(
			self.partial_cmp(permission)?,
			std::cmp::Ordering::Equal | std::cmp::Ordering::Greater
		))
	}
}

/// This function is used to deserialize a map of permissions into a Vec of
/// RolePermission objects. It is used by the RoleData struct to deserialize the
/// permissions field. The incoming map is expected to be "kind" => "value"
/// where "kind" is a string and "value" is a JSON value.
/// If the incoming map is not in the expected format, an error is returned.
/// If a "kind" is not recognized, a RolePermission::Unknown is created.
fn deserialize_permissions<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<RolePermission>, D::Error> {
	struct PermissionsVisitor;

	impl<'de> de::Visitor<'de> for PermissionsVisitor {
		type Value = Vec<RolePermission>;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str("a map of permissions")
		}

		fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
			let mut permissions = Vec::new();

			while let Some((tag, value)) = map.next_entry::<String, serde_json::Value>()? {
				permissions.push(RolePermission::from_parts(tag, value));
			}

			Ok(permissions)
		}
	}

	deserializer.deserialize_map(PermissionsVisitor)
}

/// This function is the inverse of deserialize_permissions. It is used by the
/// RoleData struct to serialize the permissions field. It takes a Vec of
/// RolePermission objects and serializes them into a map of "kind" => "value"
/// where "kind" is a string and "value" is a JSON value.
fn serialize_permissions<S: serde::Serializer>(permissions: &Vec<RolePermission>, serializer: S) -> Result<S::Ok, S::Error> {
	let mut map = serializer.serialize_map(Some(permissions.len()))?;

	for permission in permissions {
		match permission {
			RolePermission::Unknown(tag, value) => {
				map.serialize_entry(tag, value)?;
			}
			permission => {
				let (tag, value) = permission.into_parts().map_err(serde::ser::Error::custom)?;
				map.serialize_entry(&tag, &value)?;
			}
		}
	}

	map.end()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_permissions_cmp() {
		assert!(RolePermission::EmoteUpload(true).has_permission(&RolePermission::EmoteUpload(true)) == Some(true));
	}
}
