use bitmask_enum::bitmask;
use postgres_types::{FromSql, ToSql};
use crate::types::old::{
	ActiveEmoteFlagModel, ActiveEmoteModel, EmotePartialModel, EmoteSetFlagModel, EmoteSetModel, UserPartialModel,
};

use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct EmoteSet {
	pub id: ulid::Ulid,
	pub owner_id: Option<ulid::Ulid>,
	pub name: String,
	pub kind: EmoteSetKind,
	pub tags: Vec<String>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub settings: EmoteSetSettings,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl EmoteSet {
	pub fn into_old_model(
		self,
		emotes: impl IntoIterator<Item = (EmoteSetEmote, Option<EmotePartialModel>)>,
		owner: Option<UserPartialModel>,
	) -> EmoteSetModel {
		let emotes = emotes
			.into_iter()
			.map(|(emote, data)| emote.into_old_model(data))
			.collect::<Vec<_>>();

		EmoteSetModel {
			id: self.id,
			name: self.name,
			flags: {
				let mut flags = EmoteSetFlagModel::none();

				if self.kind == EmoteSetKind::Personal {
					flags |= EmoteSetFlagModel::Personal;
				}

				if self.settings.immutable {
					flags |= EmoteSetFlagModel::Immutable;
				}

				if self.settings.privileged {
					flags |= EmoteSetFlagModel::Privileged;
				}

				flags
			},
			tags: self.tags,
			immutable: self.settings.immutable,
			privileged: self.settings.privileged,
			emote_count: emotes.len() as i32,
			capacity: self.settings.capacity as i32,
			emotes,
			origins: Vec::new(),
			owner,
		}
	}
}

impl Table for EmoteSet {
	const TABLE_NAME: &'static str = "emote_sets";
}

#[derive(Debug, Clone, Default, ToSql, FromSql, PartialEq, Eq)]
#[postgres(name = "emote_set_kind")]
pub enum EmoteSetKind {
	#[default]
	#[postgres(name = "NORMAL")]
	Normal,
	#[postgres(name = "PERSONAL")]
	Personal,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct EmoteSetSettings {
	pub capacity: u32,
	pub privileged: bool,
	pub immutable: bool,
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct EmoteSetEmote {
	pub emote_set_id: ulid::Ulid,
	pub emote_id: ulid::Ulid,
	pub added_by_id: Option<ulid::Ulid>,
	pub name: String,
	pub flags: EmoteSetEmoteFlag,
	pub added_at: chrono::DateTime<chrono::Utc>,
}

#[bitmask(i32)]
pub enum EmoteSetEmoteFlag {
	ZeroWidth = 1 << 0,
	OverrideConflicts = 1 << 1,
}

impl Default for EmoteSetEmoteFlag {
	fn default() -> Self {
		EmoteSetEmoteFlag::none()
	}
}

impl postgres_types::ToSql for EmoteSetEmoteFlag {
	fn accepts(ty: &postgres_types::Type) -> bool
	where
		Self: Sized,
	{
		<i32 as postgres_types::ToSql>::accepts(ty)
	}

	fn to_sql(
		&self,
		ty: &postgres_types::Type,
		out: &mut tokio_util::bytes::BytesMut,
	) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
	where
		Self: Sized,
	{
		<i32 as postgres_types::ToSql>::to_sql(&self.bits(), ty, out)
	}

	fn to_sql_checked(
		&self,
		ty: &postgres_types::Type,
		out: &mut tokio_util::bytes::BytesMut,
	) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
		<i32 as postgres_types::ToSql>::to_sql_checked(&self.bits(), ty, out)
	}
}

impl postgres_types::FromSql<'_> for EmoteSetEmoteFlag {
	fn accepts(ty: &postgres_types::Type) -> bool
	where
		Self: Sized,
	{
		<i32 as postgres_types::FromSql>::accepts(ty)
	}

	fn from_sql(ty: &postgres_types::Type, raw: &[u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>>
	where
		Self: Sized,
	{
		<i32 as postgres_types::FromSql>::from_sql(ty, raw).map(EmoteSetEmoteFlag::from)
	}
}

impl EmoteSetEmote {
	pub fn into_old_model(self, data: Option<EmotePartialModel>) -> ActiveEmoteModel {
		ActiveEmoteModel {
			id: self.emote_id,
			actor_id: self.added_by_id,
			name: self.name,
			timestamp: self.added_at.timestamp_millis(),
			origin_id: None,
			flags: {
				let mut flags = ActiveEmoteFlagModel::none();

				if self.flags.contains(EmoteSetEmoteFlag::ZeroWidth) {
					flags |= ActiveEmoteFlagModel::ZeroWidth;
				}

				if self.flags.contains(EmoteSetEmoteFlag::OverrideConflicts) {
					flags |= ActiveEmoteFlagModel::OverrideBetterTTV
						| ActiveEmoteFlagModel::OverrideTwitchGlobal
						| ActiveEmoteFlagModel::OverrideTwitchSubscriber;
				}

				flags
			},
			data,
		}
	}
}

impl Table for EmoteSetEmote {
	const TABLE_NAME: &'static str = "emote_set_emotes";
}
