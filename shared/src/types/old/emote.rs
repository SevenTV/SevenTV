use ulid::Ulid;

use super::{ImageHost, UserPartialModel};

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct EmoteModel {
    pub id: Ulid,
    pub name: String,
    pub flags: EmoteFlagsModel,
    pub tags: Vec<String>,
    pub lifecycle: EmoteLifecycleModel,
    pub state: Vec<EmoteVersionState>,
    pub listed: bool,
    pub animated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<UserPartialModel>,
    #[serde(skip_serializing)]
    pub owner_id: Ulid,
    pub host: ImageHost,
    pub versions: Vec<EmoteVersionModel>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
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

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
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

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum EmoteLifecycleModel {
    Deleted = -1,
    Pending = 0,
    Processing = 1,
    Disabled = 2,
    Live = 3,
    Failed = -2,
}

impl serde::Serialize for EmoteLifecycleModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self as i32).serialize(serializer)
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

#[derive(Debug, Clone, Copy)]
pub enum EmoteFlagsModel {
    Private = 1 << 0,
    Authentic = 1 << 1,
    ZeroWidth = 1 << 8,
    Sexual = 1 << 16,
    Epilepsy = 1 << 17,
    Edgy = 1 << 18,
    TwitchDisallowed = 1 << 24,
}

impl serde::Serialize for EmoteFlagsModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self as i32).serialize(serializer)
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

#[derive(Debug, Clone, Copy, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmoteVersionState {
    Listed,
    AllowPersonal,
    NoPersonal,
}
