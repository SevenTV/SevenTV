use bitmask_enum::bitmask;

use super::UserId;
use crate::database::types::MongoGenericCollection;
use crate::database::MongoCollection;
use crate::typesense::types::impl_typesense_type;

#[derive(Debug, Clone, Copy, Default, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct UserEditorId {
	pub user_id: UserId,
	pub editor_id: UserId,
}

impl std::fmt::Display for UserEditorId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:{}", self.user_id, self.editor_id)
	}
}

impl std::str::FromStr for UserEditorId {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let parts = s.split(':').collect::<Vec<_>>();
		if parts.len() != 2 {
			return Err("invalid user editor id");
		}

		Ok(Self {
			user_id: parts[0].parse().map_err(|_| "invalid user id")?,
			editor_id: parts[1].parse().map_err(|_| "invalid editor id")?,
		})
	}
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "user_editors")]
#[mongo(index(fields("_id.user_id" = 1, "_id.editor_id" = 1)))]
#[mongo(index(fields("_id.editor_id" = 1, "_id.user_id" = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct UserEditor {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: UserEditorId,
	pub state: UserEditorState,
	pub notes: Option<String>,
	pub permissions: UserEditorPermissions,
	pub added_by_id: UserId,
	#[serde(with = "crate::database::serde")]
	pub added_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

macro_rules! impl_bits {
	($bits:ty) => {
		impl Default for $bits {
			fn default() -> Self {
				Self::none()
			}
		}

		impl $bits {
			pub const fn is_empty(&self) -> bool {
				self.bits() == 0
			}
		}

		impl serde::Serialize for $bits {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: serde::Serializer,
			{
				serde::Serialize::serialize(&self.bits(), serializer)
			}
		}

		impl<'de> serde::Deserialize<'de> for $bits {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				let bits = i32::deserialize(deserializer)?;
				Ok(Self::from(bits))
			}
		}
	};
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct UserEditorPermissions {
	#[serde(skip_serializing_if = "EditorEmoteSetPermission::is_empty")]
	#[serde(default)]
	pub emote_set: EditorEmoteSetPermission,
	#[serde(skip_serializing_if = "EditorEmotePermission::is_empty")]
	#[serde(default)]
	pub emote: EditorEmotePermission,
	#[serde(skip_serializing_if = "EditorUserPermission::is_empty")]
	#[serde(default)]
	pub user: EditorUserPermission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorPermission {
	User(EditorUserPermission),
	EmoteSet(EditorEmoteSetPermission),
	Emote(EditorEmotePermission),
}

impl From<EditorUserPermission> for EditorPermission {
	fn from(value: EditorUserPermission) -> Self {
		Self::User(value)
	}
}

impl From<EditorEmoteSetPermission> for EditorPermission {
	fn from(value: EditorEmoteSetPermission) -> Self {
		Self::EmoteSet(value)
	}
}

impl From<EditorEmotePermission> for EditorPermission {
	fn from(value: EditorEmotePermission) -> Self {
		Self::Emote(value)
	}
}

#[bitmask(i32)]
pub enum EditorEmoteSetPermission {
	/// Grants all permissions
	Admin = 1,
	/// Allows the user to manage the editor's emote sets
	Manage = 2,
	/// Allows the user to create new emote sets on behalf of the user
	Create = 4,
}

impl_bits!(EditorEmoteSetPermission);

#[bitmask(i32)]
pub enum EditorEmotePermission {
	/// Grants all permissions
	Admin = 1,
	/// Allows the user to manage the editor's emotes
	Manage = 2,
	/// Allows the user to create new emotes on behalf of the user
	Create = 4,
	/// Allows the user to transfer emotes to other users
	Transfer = 8,
}

impl_bits!(EditorEmotePermission);

#[bitmask(i32)]
pub enum EditorUserPermission {
	// Grants all permissions for every category
	SuperAdmin = 1,
	/// Grants all permissions
	Admin = 2,
	/// Allows the editor to manage billing information
	ManageBilling = 4,
	/// Allows the editor to manage the user's profile
	ManageProfile = 8,
	/// Allows the editor to manage the user's editors
	ManageEditors = 16,
	/// Manage personal emote set
	ManagePersonalEmoteSet = 32,
}

impl_bits!(EditorUserPermission);

impl UserEditorPermissions {
	pub fn has(&self, permission: impl Into<EditorPermission>) -> bool {
		match permission.into() {
			EditorPermission::User(permission) => self.user.contains(permission),
			EditorPermission::EmoteSet(permission) => self.emote_set.contains(permission),
			EditorPermission::Emote(permission) => self.emote.contains(permission),
		}
	}

	pub fn has_emote_set(&self, permission: EditorEmoteSetPermission) -> bool {
		self.emote_set.contains(permission)
			|| self.emote_set.contains(EditorEmoteSetPermission::Admin)
			|| self.user.contains(EditorUserPermission::SuperAdmin)
	}

	pub fn has_emote(&self, permission: EditorEmotePermission) -> bool {
		self.emote.contains(permission)
			|| self.emote.contains(EditorEmotePermission::Admin)
			|| self.user.contains(EditorUserPermission::SuperAdmin)
	}

	pub fn has_user(&self, permission: EditorUserPermission) -> bool {
		if permission.contains(EditorUserPermission::SuperAdmin) {
			return self.user.contains(EditorUserPermission::SuperAdmin);
		}

		self.user.contains(permission)
			|| self.user.contains(EditorUserPermission::Admin)
			|| self.user.contains(EditorUserPermission::SuperAdmin)
	}
}

#[derive(Debug, Clone, Default, serde_repr::Serialize_repr, serde_repr::Deserialize_repr, PartialEq, Eq)]
#[repr(i32)]
pub enum UserEditorState {
	#[default]
	Pending = 0,
	Accepted = 1,
	Rejected = 2,
}

impl_typesense_type!(UserEditorState, Int32);

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<UserEditor>()]
}
