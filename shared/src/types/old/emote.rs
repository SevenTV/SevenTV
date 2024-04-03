use bitmask_enum::bitmask;
use ulid::Ulid;

use super::{ImageHost, UserPartialModel};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct EmoteModel {
    pub id: Ulid,
    pub name: String,
    pub flags: EmoteFlagsModel,
    pub tags: Vec<String>,
    pub lifecycle: EmoteLifecycleModel,
    pub state: Vec<EmoteVersionState>,
    pub listed: bool,
    pub animated: bool,
    pub owner: Option<UserPartialModel>,
    #[serde(skip)]
    pub owner_id: Ulid,
    pub host: ImageHost,
    pub versions: Vec<EmoteVersionModel>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct EmotePartialModel {
    pub id: Ulid,
    pub name: String,
    pub flags: EmoteFlagsModel,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub lifecycle: EmoteLifecycleModel,
    pub state: Vec<EmoteVersionState>,
    pub listed: bool,
    pub animated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<UserPartialModel>,
    pub host: ImageHost,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct EmoteVersionModel {
    pub id: Ulid,
    pub name: String,
    pub description: String,
    pub lifecycle: EmoteLifecycleModel,
    pub state: Vec<EmoteVersionState>,
    pub listed: bool,
    pub animated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<ImageHost>,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(i32)]
pub enum EmoteLifecycleModel {
    Deleted = -1,
    #[default]
    Pending = 0,
    Processing = 1,
    Disabled = 2,
    Live = 3,
    Failed = -2,
}

impl serde::Serialize for EmoteLifecycleModel {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (*self as i32).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for EmoteLifecycleModel {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = i32::deserialize(deserializer)?;
        match value {
            -1 => Ok(EmoteLifecycleModel::Deleted),
            0 => Ok(EmoteLifecycleModel::Pending),
            1 => Ok(EmoteLifecycleModel::Processing),
            2 => Ok(EmoteLifecycleModel::Disabled),
            3 => Ok(EmoteLifecycleModel::Live),
            -2 => Ok(EmoteLifecycleModel::Failed),
            _ => Err(serde::de::Error::custom("invalid emote lifecycle")),
        }
    }
}

impl<'a> utoipa::ToSchema<'a> for EmoteLifecycleModel {
    fn schema() -> (&'a str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
        (
            "EmoteLifecycleModel",
            utoipa::openapi::ObjectBuilder::new()
                .schema_type(utoipa::openapi::schema::SchemaType::Integer)
                .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                    utoipa::openapi::KnownFormat::Int32,
                )))
                .into(),
        )
    }
}

#[bitmask(i32)]
pub enum EmoteFlagsModel {
    Private = 1 << 0,
    Authentic = 1 << 1,
    ZeroWidth = 1 << 8,
    Sexual = 1 << 16,
    Epilepsy = 1 << 17,
    Edgy = 1 << 18,
    TwitchDisallowed = 1 << 24,
}

impl Default for EmoteFlagsModel {
    fn default() -> Self {
        EmoteFlagsModel::none()
    }
}

impl serde::Serialize for EmoteFlagsModel {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.bits().serialize(serializer)
    }
}

impl<'a> serde::Deserialize<'a> for EmoteFlagsModel {
    fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<EmoteFlagsModel, D::Error> {
        let bits = i32::deserialize(deserializer)?;
        EmoteFlagsModel::try_from(bits).map_err(serde::de::Error::custom)
    }
}

impl<'a> utoipa::ToSchema<'a> for EmoteFlagsModel {
    fn schema() -> (&'a str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
        (
            "EmoteFlagsModel",
            utoipa::openapi::ObjectBuilder::new()
                .schema_type(utoipa::openapi::schema::SchemaType::Integer)
                .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                    utoipa::openapi::KnownFormat::Int32,
                )))
                .into(),
        )
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmoteVersionState {
    #[default]
    Listed,
    AllowPersonal,
    NoPersonal,
}
