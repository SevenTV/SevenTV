use bitmask_enum::bitmask;
use ulid::Ulid;
use super::{is_default, EmotePartialModel, UserPartialModel};

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct EmoteSetModel {
    pub id: Ulid,
    pub name: String,
    pub flags: EmoteSetFlagModel,
    pub tags: Vec<String>,
    pub immutable: bool,
    pub privileged: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub emotes: Vec<ActiveEmoteModel>,
    #[serde(skip_serializing_if = "is_default")]
    pub emote_count: i32,
    pub capacity: i32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub origins: Vec<EmoteSetOrigin>,
    pub owner: Option<UserPartialModel>,
}


#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct EmoteSetPartialModel {
    pub id: Ulid,
    pub name: String,
    pub flags: EmoteSetFlagModel,
    pub tags: Vec<String>,
    pub capacity: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<UserPartialModel>,
}

#[bitmask(i32)]
pub enum EmoteSetFlagModel {
    Immutable = 1 << 0,
    Privileged = 1 << 1,
    Personal = 1 << 2,
    Commercial = 1 << 3,
}

impl serde::Serialize for EmoteSetFlagModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl<'a> utoipa::ToSchema<'a> for EmoteSetFlagModel {
	fn schema() -> (&'a str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
		(
			"EmoteSetFlagModel",
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
pub struct ActiveEmoteModel {
    pub id: Ulid,
    pub name: String,
    pub flags: ActiveEmoteFlagModel,
    pub timestamp: i64,
    pub actor_id: Option<Ulid>,
    pub data: Option<EmotePartialModel>,
    pub origin_id: Option<Ulid>,
}

#[bitmask(i32)]
pub enum ActiveEmoteFlagModel {
    ZeroWidth = 1 << 0,
    OverrideTwitchGlobal = 1 << 16,
    OverrideTwitchSubscriber = 1 << 17,
    OverrideBetterTTV = 1 << 18,
}

impl serde::Serialize for ActiveEmoteFlagModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl<'a> utoipa::ToSchema<'a> for ActiveEmoteFlagModel {
    fn schema() -> (&'a str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
        (
            "ActiveEmoteFlagModel",
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
pub struct EmoteSetOrigin {
    pub id: Ulid,
    pub weight: i32,
    pub slices: Vec<u32>,
}
