use std::sync::Arc;

use async_graphql::{indexmap, ComplexObject, Context, ScalarType, SimpleObject};
use shared::{
	database::{
		EmoteActivity, EmoteActivityData, EmoteActivityKind, EmoteId, EmoteSetActivity, EmoteSetActivityData,
		EmoteSetActivityKind, EmoteSetId, EmoteSettingsChange, Id, UserId,
	},
	old_types::{EmoteFlagsModel, ObjectId, UserObjectId},
};

use crate::{global::Global, http::error::ApiError};

use super::users::UserPartial;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/audit.gql

#[derive(Debug, SimpleObject)]
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

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl AuditLog {
	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		UserPartial::load_from_db(global, self.actor_id.id()).await
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
			target_id: EmoteId::from(activity.emote_id).cast().into(),
			target_kind: 2,
			created_at: activity.timestamp,
			changes,
			reason: String::new(),
		}
	}

	pub fn from_db_emote_set(activity: EmoteSetActivity) -> Self {
		let changes = activity
			.data
			.and_then(AuditLogChange::from_db_emote_set)
			.map(|c| vec![c])
			.unwrap_or_default();

		Self {
			id: Id::<()>::with_timestamp_ms(activity.timestamp.unix_timestamp() * 1000).into(),
			actor_id: activity.actor_id.map(UserId::from).unwrap_or(UserId::nil()).into(),
			kind: activity.kind.into(),
			target_id: EmoteSetId::from(activity.emote_set_id).cast().into(),
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

	pub fn from_db_emote_set(data: EmoteSetActivityData) -> Option<Self> {
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
					added: added.into_iter().map(|id| ArbitraryMap::Emote { emote_id: id }).collect(),
					removed: removed.into_iter().map(|id| ArbitraryMap::Emote { emote_id: id }).collect(),
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
		emote_id: EmoteId,
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
			Self::Emote { emote_id } => {
				indexmap::indexmap! {
					async_graphql::Name::new("emote_id") => async_graphql::Value::String(emote_id.to_string()),
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
