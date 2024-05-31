use shared::{database::{User, UserConnection, UserEditor, UserEditorState}, old_types::{CosmeticBadgeModel, CosmeticPaintModel, UserPartialModel, UserStyle, UserTypeModel}};
use shared::database::{RoleId, UserId};

use crate::http::v3::types::UserEditorModelPermission;

use super::{is_default, EmoteSetPartialModel, UserConnectionModel};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user.model.go#L15
pub struct UserModel {
	pub id: UserId,
	#[serde(skip_serializing_if = "is_default")]
	pub user_type: UserTypeModel,
	pub username: String,
	pub display_name: String,
	#[serde(skip_serializing_if = "is_default")]
	pub created_at: u64,
	#[serde(skip_serializing_if = "String::is_empty")]
	pub avatar_url: String,
	#[serde(skip_serializing_if = "String::is_empty")]
	pub biography: String,
	pub style: UserStyle,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub emote_sets: Vec<EmoteSetPartialModel>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub editors: Vec<UserEditorModel>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub roles: Vec<RoleId>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub connections: Vec<UserConnectionModel>,
}

impl UserModel {
	pub fn from_db(
		value: User,
		connections: Vec<UserConnection>,
		paint: Option<CosmeticPaintModel>,
		badge: Option<CosmeticBadgeModel>,
		emote_sets: Vec<EmoteSetPartialModel>,
		editors: Vec<UserEditorModel>,
		cdn_base_url: &str,
	) -> Self {
		let created_at = value.id.timestamp_ms();
		let partial = UserPartialModel::from_db(value, connections, paint, badge, cdn_base_url);

		Self {
			id: partial.id,
			user_type: partial.user_type,
			username: partial.username,
			display_name: partial.display_name,
			created_at,
			avatar_url: partial.avatar_url,
			biography: String::new(),
			style: partial.style,
			emote_sets,
			editors,
			roles: partial.role_ids,
			connections: partial
				.connections
				.into_iter()
				.map(|p| UserConnectionModel {
					id: p.id,
					platform: p.platform,
					username: p.username,
					display_name: p.display_name,
					linked_at: p.linked_at,
					emote_capacity: p.emote_capacity,
					emote_set_id: p.emote_set_id,
					emote_set: None,
					user: None,
				})
				.collect(),
		}
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user.model.go#L171
pub struct UserEditorModel {
	pub id: UserId,
	pub permissions: UserEditorModelPermission,
	pub visible: bool,
	pub added_at: u64,
}

impl UserEditorModel {
	pub fn from_db(value: UserEditor) -> Option<Self> {
		if value.state != UserEditorState::Accepted {
			return None;
		}

		Some(Self {
			id: value.editor_id,
			added_at: value.id.timestamp_ms(),
			permissions: UserEditorModelPermission::ModifyEmotes | UserEditorModelPermission::ManageEmoteSets,
			visible: true,
		})
	}
}
