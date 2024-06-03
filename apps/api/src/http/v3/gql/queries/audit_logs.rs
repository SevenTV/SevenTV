use async_graphql::SimpleObject;
use shared::old_types::{ObjectId, UserObjectId};

use super::users::UserPartial;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/audit.gql

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct AuditLog {
	id: ObjectId<()>,
	actor: UserPartial,
	actor_id: UserObjectId,
	kind: u32,
	target_id: ObjectId<()>,
	// created_at
	changes: Vec<AuditLogChange>,
	reason: String,
}

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct AuditLogChange {
	format: u32,
	key: String,
	value: ArbitraryMap,
	array_value: AuditLogChangeArray,
}

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
