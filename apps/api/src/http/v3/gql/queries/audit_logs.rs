use std::sync::Arc;

use async_graphql::{ComplexObject, Context, SimpleObject};
use shared::{
	database::{EmoteActivity, EmoteActivityData, EmoteActivityKind, EmoteId, EmoteSettingsChange, Id, UserId},
	old_types::{EmoteFlagsModel, ObjectId, UserObjectId},
};

use crate::{global::Global, http::error::ApiError};

use super::users::UserPartial;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/audit.gql

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct AuditLog {
	id: ObjectId<()>,
	// actor
	actor_id: UserObjectId,
	kind: AuditLogKind,
	target_id: ObjectId<()>,
	target_kind: u32,
	created_at: time::OffsetDateTime,
	changes: Vec<AuditLogChange>,
	reason: String,
}

// https://github.com/SevenTV/Common/blob/master/structures/v3/type.audit.go#L21
#[derive(Debug, Clone, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(u32)]
pub enum AuditLogKind {
	CreateEmote = 1,
	DeleteEmote = 2,
	DisableEmote = 3,
	UpdateEmote = 4,
	MergeEmote = 5,
	UndoDeleteEmote = 6,
	EnableEmote = 7,
	ProcessEmote = 8,

	SignUserToken = 20,
	SignCsrfToken = 21,
	RejectedAccess = 26,

	CreateUser = 30,
	DeleteUser = 31,
	BanUser = 32,
	EditUser = 33,
	UnbanUser = 36,

	CreateEmoteSet = 70,
	UpdateEmoteSet = 71,
	DeleteEmoteSet = 72,

	CreateReport = 80,
	UpdateReport = 81,

	ReadMessage = 90,
}

async_graphql::scalar!(AuditLogKind);

impl From<EmoteActivityKind> for AuditLogKind {
	fn from(value: EmoteActivityKind) -> Self {
		match value {
			EmoteActivityKind::Upload => Self::CreateEmote,
			EmoteActivityKind::Process => Self::ProcessEmote,
			EmoteActivityKind::Edit => Self::UpdateEmote,
			EmoteActivityKind::Merge => Self::MergeEmote,
			EmoteActivityKind::Delete => Self::DeleteEmote,
		}
	}
}

impl AuditLog {
	pub fn from_db_emote(activity: EmoteActivity) -> Self {
		let changes = activity
			.data
			.and_then(AuditLogChange::from_db)
			.map(|c| vec![c])
			.unwrap_or_default();

		Self {
			id: Id::<()>::new().into(),
			actor_id: activity.actor_id.map(UserId::from).unwrap_or(UserId::nil()).into(),
			kind: activity.kind.into(),
			target_id: EmoteId::from(activity.emote_id).cast().into(),
			target_kind: 2,
			created_at: activity.timestamp,
			changes,
			reason: String::new(),
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl AuditLog {
	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		UserPartial::load_from_db(global, self.actor_id.id()).await
	}
}

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct AuditLogChange {
	format: AuditLogChangeFormat,
	key: String,
	value: ArbitraryMap,
	array_value: AuditLogChangeArray,
}

impl AuditLogChange {
	pub fn from_db(data: EmoteActivityData) -> Option<Self> {
		let mut value = ArbitraryMap::default();
		value.0.insert(async_graphql::Name::new("p"), 0.into());

		match data {
			EmoteActivityData::ChangeName { old, new } => {
				value.0.insert(async_graphql::Name::new("n"), new.into());
				value.0.insert(async_graphql::Name::new("o"), old.into());

				Some(Self {
					format: AuditLogChangeFormat::SingleValue,
					key: "name".to_string(),
					value,
					..Default::default()
				})
			}
			EmoteActivityData::ChangeOwner { old, new } => {
				value.0.insert(async_graphql::Name::new("n"), new.to_string().into());
				value.0.insert(async_graphql::Name::new("o"), old.to_string().into());

				Some(Self {
					format: AuditLogChangeFormat::SingleValue,
					key: "owner_id".to_string(),
					value,
					..Default::default()
				})
			}
			EmoteActivityData::ChangeTags { added, removed } => {
				value.0.insert(async_graphql::Name::new("n"), added.into());
				value.0.insert(async_graphql::Name::new("o"), removed.into());

				Some(Self {
					format: AuditLogChangeFormat::SingleValue,
					key: "tags".to_string(),
					value,
					..Default::default()
				})
			}
			EmoteActivityData::ChangeSettings { old, new } => {
				if (new.approved_personal.is_some() && old.approved_personal.is_some())
					|| (new.public_listed.is_some() && old.public_listed.is_some())
				{
					value
						.0
						.insert(async_graphql::Name::new("n"), ArbitraryMap::from(new).0.into());
					value
						.0
						.insert(async_graphql::Name::new("o"), ArbitraryMap::from(old).0.into());

					Some(Self {
						format: AuditLogChangeFormat::ArrayChange,
						key: "versions".to_string(),
						array_value: AuditLogChangeArray {
							updated: vec![value],
							..Default::default()
						},
						..Default::default()
					})
				} else {
					let old_flags = emote_settings_change_to_flags(old);
					let new_flags = emote_settings_change_to_flags(new);

					value.0.insert(async_graphql::Name::new("n"), new_flags.bits().into());
					value.0.insert(async_graphql::Name::new("o"), old_flags.bits().into());

					Some(Self {
						format: AuditLogChangeFormat::ArrayChange,
						key: "flags".to_string(),
						value,
						..Default::default()
					})
				}
			}
			_ => None,
		}
	}
}

#[derive(Debug, Default, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum AuditLogChangeFormat {
	#[default]
	SingleValue = 1,
	ArrayChange = 2,
}

async_graphql::scalar!(AuditLogChangeFormat);

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct AuditLogChangeArray {
	added: Vec<ArbitraryMap>,
	removed: Vec<ArbitraryMap>,
	updated: Vec<ArbitraryMap>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct ArbitraryMap(async_graphql::indexmap::IndexMap<async_graphql::Name, async_graphql::Value>);

async_graphql::scalar!(ArbitraryMap);

impl From<EmoteSettingsChange> for ArbitraryMap {
	fn from(value: EmoteSettingsChange) -> Self {
		let mut map = Self::default();

		if let Some(v) = value.public_listed {
			map.0.insert(async_graphql::Name::new("listed"), v.into());
		}

		if let Some(v) = value.approved_personal {
			map.0.insert(async_graphql::Name::new("allow_personal"), v.into());
		}

		map
	}
}

fn emote_settings_change_to_flags(value: EmoteSettingsChange) -> EmoteFlagsModel {
	let mut flags = EmoteFlagsModel::default();

	if let Some(true) = value.private {
		flags |= EmoteFlagsModel::Private;
	}

	if let Some(true) = value.nsfw {
		flags |= EmoteFlagsModel::Sexual;
	}

	if let Some(true) = value.default_zero_width {
		flags |= EmoteFlagsModel::ZeroWidth;
	}

	flags
}
