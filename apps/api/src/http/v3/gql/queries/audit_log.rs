use std::str::FromStr;
use std::sync::Arc;

use async_graphql::{indexmap, ComplexObject, Context, ScalarType, SimpleObject};
use shared::database::emote::EmoteFlags;
use shared::database::stored_event::{
	ImageProcessorEvent, StoredEventEmoteData, StoredEventEmoteSetData, StoredEventUserEditorData,
};
use shared::database::user::UserId;
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::EmoteFlagsModel;

use super::user::UserPartial;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

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
	created_at: chrono::DateTime<chrono::Utc>,
	changes: Vec<AuditLogChange>,
	reason: String,
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl AuditLog {
	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(global
			.user_loader
			.load_fast(global, self.actor_id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.map(|u| UserPartial::from_db(global, u))
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

impl AuditLog {
	pub fn from_db(audit_log: shared::database::stored_event::StoredEvent) -> Option<Self> {
		let actor_id = audit_log
			.actor_id
			.map(UserId::from)
			.unwrap_or_else(|| UserId::from_str("01FRG0ZGSR00084PQ73P1BYDX8").unwrap()) // system user
			.into();

		let (kind, target_id, target_kind, changes) = match audit_log.data {
			shared::database::stored_event::StoredEventData::Emote {
				target_id,
				data: StoredEventEmoteData::Upload,
			} => (AuditLogKind::CreateEmote, target_id.into(), 2, vec![]),
			shared::database::stored_event::StoredEventData::Emote {
				target_id,
				data: StoredEventEmoteData::Process {
					event: ImageProcessorEvent::Success,
				},
			} => (AuditLogKind::ProcessEmote, target_id.into(), 2, vec![]),
			shared::database::stored_event::StoredEventData::Emote {
				target_id,
				data: StoredEventEmoteData::Delete,
			} => (AuditLogKind::DeleteEmote, target_id.into(), 2, vec![]),
			shared::database::stored_event::StoredEventData::Emote { target_id, data } => (
				AuditLogKind::UpdateEmote,
				target_id.into(),
				2,
				vec![AuditLogChange::from_db_emote(data)?],
			),
			shared::database::stored_event::StoredEventData::EmoteSet {
				target_id,
				data: StoredEventEmoteSetData::Create,
			} => (AuditLogKind::CreateEmoteSet, target_id.into(), 3, vec![]),
			shared::database::stored_event::StoredEventData::EmoteSet {
				target_id,
				data: StoredEventEmoteSetData::Delete,
			} => (AuditLogKind::DeleteEmoteSet, target_id.into(), 3, vec![]),
			shared::database::stored_event::StoredEventData::EmoteSet { target_id, data } => (
				AuditLogKind::UpdateEmoteSet,
				target_id.into(),
				3,
				vec![AuditLogChange::from_db_emote_set(data, actor_id, audit_log.id.timestamp())?],
			),
			shared::database::stored_event::StoredEventData::UserEditor { target_id, data } => (
				AuditLogKind::EditUser,
				target_id.user_id.into(),
				1,
				vec![AuditLogChange::from_db_user_editor(data, audit_log.id.timestamp())?],
			),
			_ => return None,
		};

		Some(Self {
			id: audit_log.id.into(),
			actor_id,
			kind,
			target_id,
			target_kind,
			created_at: audit_log.id.timestamp(),
			changes,
			reason: String::new(),
		})
	}
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct EmoteVersionStateChange {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub listed: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub allow_personal: Option<bool>,
}

impl From<EmoteFlags> for EmoteVersionStateChange {
	fn from(value: EmoteFlags) -> Self {
		let allow_personal = match (
			value.contains(EmoteFlags::ApprovedPersonal),
			value.contains(EmoteFlags::DeniedPersonal),
		) {
			(true, false) => Some(true),
			(false, true) => Some(false),
			_ => None,
		};

		Self {
			listed: Some(value.contains(EmoteFlags::PublicListed)),
			allow_personal,
		}
	}
}

impl AuditLogChange {
	pub fn from_db_emote(data: StoredEventEmoteData) -> Option<Self> {
		match data {
			StoredEventEmoteData::ChangeName { old, new } => Some(Self {
				format: AuditLogChangeFormat::SingleValue,
				key: "name".to_string(),
				value: Some(ArbitraryMap::StringValue { n: new, o: old, p: 0 }),
				array_value: None,
			}),
			StoredEventEmoteData::ChangeOwner { old, new } => Some(Self {
				format: AuditLogChangeFormat::SingleValue,
				key: "owner_id".to_string(),
				value: Some(ArbitraryMap::StringValue {
					n: new.to_string(),
					o: old.to_string(),
					p: 0,
				}),
				array_value: None,
			}),
			StoredEventEmoteData::ChangeTags { old, new } => Some(Self {
				format: AuditLogChangeFormat::SingleValue,
				key: "tags".to_string(),
				value: Some(ArbitraryMap::StringVecValue { n: new, o: old, p: 0 }),
				array_value: None,
			}),
			StoredEventEmoteData::ChangeFlags { old, new } => {
				if new.contains(EmoteFlags::ApprovedPersonal) != old.contains(EmoteFlags::ApprovedPersonal)
					|| new.contains(EmoteFlags::DeniedPersonal) != old.contains(EmoteFlags::DeniedPersonal)
					|| new.contains(EmoteFlags::PublicListed) != old.contains(EmoteFlags::PublicListed)
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
					let old_flags: EmoteFlagsModel = old.into();
					let new_flags: EmoteFlagsModel = new.into();

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
		data: StoredEventEmoteSetData,
		actor_id: GqlObjectId,
		timestamp: chrono::DateTime<chrono::Utc>,
	) -> Option<Self> {
		match data {
			StoredEventEmoteSetData::ChangeName { old, new } => Some(Self {
				format: AuditLogChangeFormat::SingleValue,
				key: "name".to_string(),
				value: Some(ArbitraryMap::StringValue { n: new, o: old, p: 0 }),
				array_value: None,
			}),
			StoredEventEmoteSetData::AddEmote { emote_id, alias } => Some(Self {
				format: AuditLogChangeFormat::ArrayValue,
				key: "emotes".to_string(),
				value: None,
				array_value: Some(AuditLogChangeArray {
					added: vec![ArbitraryMap::Emote {
						id: emote_id.into(),
						actor_id,
						flags: EmoteFlagsModel::none(),
						name: alias,
						timestamp,
					}],
					removed: vec![],
					updated: vec![],
				}),
			}),
			StoredEventEmoteSetData::RemoveEmote { emote_id } => {
				Some(Self {
					format: AuditLogChangeFormat::ArrayValue,
					key: "emotes".to_string(),
					value: None,
					array_value: Some(AuditLogChangeArray {
						added: vec![],
						removed: vec![ArbitraryMap::Emote {
							id: emote_id.into(),
							actor_id,
							flags: EmoteFlagsModel::none(),
							name: "".to_string(), // TODO: get the emote name
							timestamp,
						}],
						updated: vec![],
					}),
				})
			}
			StoredEventEmoteSetData::RenameEmote {
				emote_id,
				old_alias: old_name,
				new_alias: new_name,
			} => Some(Self {
				format: AuditLogChangeFormat::ArrayValue,
				key: "emotes".to_string(),
				value: None,
				array_value: Some(AuditLogChangeArray {
					added: vec![],
					removed: vec![],
					updated: vec![ArbitraryMap::Nested {
						o: Box::new(ArbitraryMap::Emote {
							id: emote_id.into(),
							actor_id,
							flags: EmoteFlagsModel::none(),
							name: old_name,
							timestamp,
						}),
						n: Box::new(ArbitraryMap::Emote {
							id: emote_id.into(),
							actor_id,
							flags: EmoteFlagsModel::none(),
							name: new_name,
							timestamp,
						}),
						p: 0,
					}],
				}),
			}),
			_ => None,
		}
	}

	pub fn from_db_user_editor(data: StoredEventUserEditorData, timestamp: chrono::DateTime<chrono::Utc>) -> Option<Self> {
		match data {
			StoredEventUserEditorData::AddEditor { editor_id } => Some(Self {
				format: AuditLogChangeFormat::ArrayValue,
				key: "editors".to_string(),
				value: None,
				array_value: Some(AuditLogChangeArray {
					added: vec![ArbitraryMap::Editor {
						id: editor_id.into(),
						added_at: timestamp,
						permissions: 0,
						visible: true,
					}],
					removed: vec![],
					updated: vec![],
				}),
			}),
			StoredEventUserEditorData::RemoveEditor { editor_id } => Some(Self {
				format: AuditLogChangeFormat::ArrayValue,
				key: "editors".to_string(),
				value: None,
				array_value: Some(AuditLogChangeArray {
					added: vec![],
					removed: vec![ArbitraryMap::Editor {
						id: editor_id.into(),
						added_at: timestamp,
						permissions: 0,
						visible: true,
					}],
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
		timestamp: chrono::DateTime<chrono::Utc>,
	},
	EmoteVersionState {
		n: EmoteVersionStateChange,
		o: EmoteVersionStateChange,
		p: u32,
	},
	Editor {
		id: GqlObjectId,
		added_at: chrono::DateTime<chrono::Utc>,
		permissions: u32,
		visible: bool,
	},
	Nested {
		n: Box<ArbitraryMap>,
		o: Box<ArbitraryMap>,
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
			Self::Editor {
				id,
				added_at,
				permissions,
				visible,
			} => {
				indexmap::indexmap! {
					async_graphql::Name::new("id") => async_graphql::Value::String(id.to_string()),
					async_graphql::Name::new("added_at") => async_graphql::Value::String(added_at.to_string()),
					async_graphql::Name::new("permissions") => async_graphql::Value::Number(async_graphql::Number::from(*permissions)),
					async_graphql::Name::new("visible") => async_graphql::Value::Boolean(*visible),
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
			Self::Nested { n, o, p } => {
				let n = n.to_value();
				let o = o.to_value();

				indexmap::indexmap! {
					async_graphql::Name::new("n") => n,
					async_graphql::Name::new("o") => o,
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
