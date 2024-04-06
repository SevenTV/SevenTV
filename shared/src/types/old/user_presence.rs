use ulid::Ulid;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user-presence.model.go#L9
pub struct PresenceModel {
	pub id: Ulid,
	pub user_id: Ulid,
	pub timestamp: i64,
	pub ttl: i64,
	pub kind: PresenceKind,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user-presence.model.go#L19
pub enum PresenceKind {
	#[default]
	UserPresenceKindUnknown = 0,
	UserPresenceKindChannel = 1,
	UserPresenceKindWebPage = 2,
}

impl serde::Serialize for PresenceKind {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		(*self as u8).serialize(serializer)
	}
}

impl<'de> serde::Deserialize<'de> for PresenceKind {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let value = u8::deserialize(deserializer)?;
		match value {
			0 => Ok(PresenceKind::UserPresenceKindUnknown),
			1 => Ok(PresenceKind::UserPresenceKindChannel),
			2 => Ok(PresenceKind::UserPresenceKindWebPage),
			_ => Err(serde::de::Error::custom("invalid presence kind")),
		}
	}
}

impl<'a> utoipa::ToSchema<'a> for PresenceKind {
	fn schema() -> (&'a str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
		(
			"PresenceKind",
			utoipa::openapi::ObjectBuilder::new()
				.schema_type(utoipa::openapi::schema::SchemaType::Integer)
				.format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
					utoipa::openapi::KnownFormat::Int32,
				)))
				.into(),
		)
	}
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct UserPresenceWriteResponse {
	pub ok: bool,
	pub presence: PresenceModel,
}
