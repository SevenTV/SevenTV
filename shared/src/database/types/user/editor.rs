use bitmask_enum::bitmask;

use super::UserId;
use crate::database::types::GenericCollection;
use crate::database::Collection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct UserEditorId {}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserEditor {
	#[serde(rename = "_id.user_id")]
	pub user_id: UserId,
	#[serde(rename = "_id.editor_id")]
	pub editor_id: UserId,
	pub state: UserEditorState,
	pub notes: Option<String>,
	pub permissions: UserEditorPermissions,
	pub added_by_id: UserId,
	pub added_at: chrono::DateTime<chrono::Utc>,
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
#[repr(u8)]
pub enum UserEditorState {
	#[default]
	Pending = 0,
	Accepted = 1,
	Rejected = 2,
}

impl Collection for UserEditor {
	const COLLECTION_NAME: &'static str = "user_editors";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"_id.user_id": 1,
					"_id.editor_id": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"_id.editor_id": 1,
					"_id.user_id": 1,
				})
				.build(),
		]
	}
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<UserEditor>()]
}
