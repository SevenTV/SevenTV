use shared::database::{Id, UserId};

use super::users::UserPartial;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/audit.gql

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct AuditLog {
    id: Id<()>,
    actor: UserPartial,
    actor_id: UserId,
    // ...
}
