use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct Role {
	pub id: ulid::Ulid,
	pub name: String,
	pub description: Option<String>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: RoleData,
	pub priority: i16,
	pub hoist: bool,
	pub color: i32,
	pub tags: Vec<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Table for Role {
	const TABLE_NAME: &'static str = "roles";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct RoleData {
	// TODO: permissions
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct RoleBadge {
	pub role_id: ulid::Ulid,
	pub badge_id: ulid::Ulid,
}

impl Table for RoleBadge {
	const TABLE_NAME: &'static str = "role_badges";
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct RoleEmoteSet {
	pub role_id: ulid::Ulid,
	pub emote_set_id: ulid::Ulid,
}

impl Table for RoleEmoteSet {
	const TABLE_NAME: &'static str = "role_emote_sets";
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct RolePaint {
	pub role_id: ulid::Ulid,
	pub paint_id: ulid::Ulid,
}

impl Table for RolePaint {
	const TABLE_NAME: &'static str = "role_paints";
}
