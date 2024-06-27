use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::{indexmap, ComplexObject, Context, ScalarType, SimpleObject};
use shared::database::activity::{
	EmoteActivity, EmoteActivityData, EmoteActivityKind, EmoteSetActivity, EmoteSetActivityData, EmoteSetActivityKind,
	EmoteSettingsChange,
};
use shared::database::emote::{Emote, EmoteId};
use shared::database::emote_set::EmoteSetId;
use shared::database::user::UserId;
use shared::database::Id;
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::EmoteFlagsModel;

use super::user::UserPartial;
use crate::global::Global;
use crate::http::error::ApiError;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/audit.gql

#[derive(Debug, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct AuditLog {
	id: GqlObjectId,
	// actor
	actor_id: GqlObjectId,
	kind: AuditLogKind,
	target_id: GqlObjectId,
	target_kind: u32,
	created_at: time::OffsetDateTime,
	changes: Vec<AuditLogChange>,
	reason: String,
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl AuditLog {
	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(global
			.user_by_id_loader()
			.load(self.actor_id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|u| UserPartial::from_db(global, u.into()))
			.unwrap_or_else(UserPartial::deleted_user))
	}
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

impl From<EmoteSetActivityKind> for AuditLogKind {
	fn from(value: EmoteSetActivityKind) -> Self {
		match value {
			EmoteSetActivityKind::Create => Self::CreateEmoteSet,
			EmoteSetActivityKind::Edit => Self::UpdateEmoteSet,
			EmoteSetActivityKind::Delete => Self::DeleteEmoteSet,
		}
	}
}

impl AuditLog {
	pub fn from_db_emote(activity: EmoteActivity) -> Self {
		let changes = activity
			.data
			.and_then(AuditLogChange::from_db_emote)
			.map(|c| vec![c])
			.unwrap_or_default();

		Self {
			id: Id::<()>::with_timestamp_ms(activity.timestamp.unix_timestamp() * 1000).into(),
			actor_id: activity.actor_id.map(UserId::from).unwrap_or(UserId::nil()).into(),
			kind: activity.kind.into(),
			target_id: EmoteId::from(activity.emote_id).into(),
			target_kind: 2,
			created_at: activity.timestamp,
			changes,
			reason: String::new(),
		}
	}

	pub fn from_db_emote_set(activity: EmoteSetActivity, emotes: &HashMap<EmoteId, Emote>) -> Self {
		let actor_id = activity.actor_id.map(UserId::from).unwrap_or(UserId::nil()).into();

		let changes = activity
			.data
			.and_then(|c| AuditLogChange::from_db_emote_set(c, actor_id, activity.timestamp, emotes))
			.map(|c| vec![c])
			.unwrap_or_default();

		Self {
			id: Id::<()>::with_timestamp_ms(activity.timestamp.unix_timestamp() * 1000).into(),
			actor_id,
			kind: activity.kind.into(),
			target_id: EmoteSetId::from(activity.emote_set_id).into(),
			target_kind: 3,
			created_at: activity.timestamp,
			changes,
			reason: String::new(),
		}
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct EmoteVersionStateChange {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub listed: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub allow_personal: Option<bool>,
}

impl From<EmoteSettingsChange> for EmoteVersionStateChange {
	fn from(value: EmoteSettingsChange) -> Self {
		Self {
			listed: value.public_listed,
			allow_personal: value.approved_personal,
		}
	}
}

impl AuditLogChange {
	pub fn from_db_emote(data: EmoteActivityData) -> Option<Self> {
		match data {
			EmoteActivityData::ChangeName { old, new } => Some(Self {
				format: AuditLogChangeFormat::SingleValue,
				key: "name".to_string(),
				value: Some(ArbitraryMap::StringValue { n: new, o: old, p: 0 }),
				array_value: None,
			}),
			EmoteActivityData::ChangeOwner { old, new } => Some(Self {
				format: AuditLogChangeFormat::SingleValue,
				key: "owner_id".to_string(),
				value: Some(ArbitraryMap::StringValue {
					n: new.to_string(),
					o: old.to_string(),
					p: 0,
				}),
				array_value: None,
			}),
			EmoteActivityData::ChangeTags { old, new } => Some(Self {
				format: AuditLogChangeFormat::SingleValue,
				key: "tags".to_string(),
				value: Some(ArbitraryMap::StringVecValue { n: new, o: old, p: 0 }),
				array_value: None,
			}),
			EmoteActivityData::ChangeSettings { old, new } => {
				if (new.approved_personal.is_some() && old.approved_personal.is_some())
					|| (new.public_listed.is_some() && old.public_listed.is_some())
				{
					Some(Self {
						format: AuditLogChangeFormat::ArrayValue,
						key: "versions".to_string(),
						value: None,
						array_value: Some(AuditLogChangeArray {
							added: vec![],
							removed: vec![],
							updated: vec![ArbitraryMap::EmoteVersionState {
								n: EmoteVersionStateChange::from(new),
								o: EmoteVersionStateChange::from(old),
								p: 0,
							}],
						}),
					})
				} else {
					let old_flags = emote_settings_change_to_flags(old);
					let new_flags = emote_settings_change_to_flags(new);

					Some(Self {
						format: AuditLogChangeFormat::SingleValue,
						key: "flags".to_string(),
						value: Some(ArbitraryMap::NumberValue {
							n: new_flags.bits(),
							o: old_flags.bits(),
							p: 0,
						}),
						array_value: None,
					})
				}
			}
			_ => None,
		}
	}

	pub fn from_db_emote_set(
		data: EmoteSetActivityData,
		actor_id: GqlObjectId,
		timestamp: time::OffsetDateTime,
		emotes: &HashMap<EmoteId, Emote>,
	) -> Option<Self> {
		match data {
			EmoteSetActivityData::ChangeName { old, new } => Some(Self {
				format: AuditLogChangeFormat::SingleValue,
				key: "name".to_string(),
				value: Some(ArbitraryMap::StringValue { n: new, o: old, p: 0 }),
				array_value: None,
			}),
			EmoteSetActivityData::ChangeSettings { .. } => None,
			EmoteSetActivityData::ChangeEmotes { added, removed } => Some(Self {
				format: AuditLogChangeFormat::ArrayValue,
				key: "emotes".to_string(),
				value: None,
				array_value: Some(AuditLogChangeArray {
					added: added
						.into_iter()
						.filter_map(|id| {
							let emote = emotes.get(&id)?;
							Some(ArbitraryMap::Emote {
								id: id.into(),
								actor_id,
								flags: EmoteFlagsModel::none(),
								name: emote.default_name.clone(),
								timestamp,
							})
						})
						.collect(),
					removed: removed
						.into_iter()
						.filter_map(|id| {
							let emote = emotes.get(&id)?;
							Some(ArbitraryMap::Emote {
								id: id.into(),
								actor_id,
								flags: EmoteFlagsModel::none(),
								name: emote.default_name.clone(),
								timestamp,
							})
						})
						.collect(),
					updated: vec![],
				}),
			}),
			_ => None,
		}
	}
}

#[derive(Debug, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct AuditLogChange {
	format: AuditLogChangeFormat,
	key: String,
	value: Option<ArbitraryMap>,
	array_value: Option<AuditLogChangeArray>,
}

#[derive(Debug, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(u32)]
pub enum AuditLogChangeFormat {
	SingleValue = 1,
	ArrayValue = 2,
}

async_graphql::scalar!(AuditLogChangeFormat);

#[derive(Debug, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct AuditLogChangeArray {
	added: Vec<ArbitraryMap>,
	removed: Vec<ArbitraryMap>,
	updated: Vec<ArbitraryMap>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum ArbitraryMap {
	Emote {
		id: GqlObjectId,
		actor_id: GqlObjectId,
		flags: EmoteFlagsModel,
		name: String,
		timestamp: time::OffsetDateTime,
	},
	EmoteVersionState {
		n: EmoteVersionStateChange,
		o: EmoteVersionStateChange,
		p: u32,
	},
	StringVecValue {
		n: Vec<String>,
		o: Vec<String>,
		p: u32,
	},
	NumberValue {
		n: u32,
		o: u32,
		p: u32,
	},
	StringValue {
		n: String,
		o: String,
		p: u32,
	},
}

#[async_graphql::Scalar]
impl ScalarType for ArbitraryMap {
	fn parse(_: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
		unimplemented!()
	}

	fn to_value(&self) -> async_graphql::Value {
		let map = match self {
			Self::Emote {
				id,
				actor_id,
				flags,
				name,
				timestamp,
			} => {
				indexmap::indexmap! {
					async_graphql::Name::new("id") => async_graphql::Value::String(id.to_string()),
					async_graphql::Name::new("actor_id") => async_graphql::Value::String(actor_id.to_string()),
					async_graphql::Name::new("flags") => async_graphql::Value::Number(async_graphql::Number::from(flags.bits())),
					async_graphql::Name::new("name") => async_graphql::Value::String(name.clone()),
					async_graphql::Name::new("timestamp") => async_graphql::Value::String(timestamp.to_string()),
				}
			}
			Self::EmoteVersionState { n, o, p } => {
				let mut new: indexmap::IndexMap<async_graphql::Name, async_graphql::Value> = indexmap::IndexMap::new();
				if let Some(listed) = n.listed {
					new.insert(async_graphql::Name::new("listed"), listed.into());
				}
				if let Some(allow_personal) = n.allow_personal {
					new.insert(async_graphql::Name::new("allow_personal"), allow_personal.into());
				}

				let mut old: indexmap::IndexMap<async_graphql::Name, async_graphql::Value> = indexmap::IndexMap::new();
				if let Some(listed) = o.listed {
					old.insert(async_graphql::Name::new("listed"), listed.into());
				}
				if let Some(allow_personal) = o.allow_personal {
					old.insert(async_graphql::Name::new("allow_personal"), allow_personal.into());
				}

				indexmap::indexmap! {
					async_graphql::Name::new("n") => async_graphql::Value::Object(new),
					async_graphql::Name::new("o") => async_graphql::Value::Object(old),
					async_graphql::Name::new("p") => async_graphql::Value::Number(async_graphql::Number::from(*p)),
				}
			}
			Self::StringVecValue { n, o, p } => {
				let n = n.iter().map(|v| async_graphql::Value::String(v.clone())).collect();
				let o = o.iter().map(|v| async_graphql::Value::String(v.clone())).collect();

				indexmap::indexmap! {
					async_graphql::Name::new("n") => async_graphql::Value::List(n),
					async_graphql::Name::new("o") => async_graphql::Value::List(o),
					async_graphql::Name::new("p") => async_graphql::Value::Number(async_graphql::Number::from(*p)),
				}
			}
			Self::NumberValue { n, o, p } => {
				indexmap::indexmap! {
					async_graphql::Name::new("n") => async_graphql::Value::Number(async_graphql::Number::from(*n)),
					async_graphql::Name::new("o") => async_graphql::Value::Number(async_graphql::Number::from(*o)),
					async_graphql::Name::new("p") => async_graphql::Value::Number(async_graphql::Number::from(*p)),
				}
			}
			Self::StringValue { n, o, p } => {
				indexmap::indexmap! {
					async_graphql::Name::new("n") => async_graphql::Value::String(n.clone()),
					async_graphql::Name::new("o") => async_graphql::Value::String(o.clone()),
					async_graphql::Name::new("p") => async_graphql::Value::Number(async_graphql::Number::from(*p)),
				}
			}
		};

		async_graphql::Value::Object(map)
	}
}
