use async_graphql::MergedObject;

// https://github.com/SevenTV/API/tree/main/internal/api/gql/v3/schema

pub mod audit_log;
pub mod cosmetic;
pub mod emote;
pub mod emote_set;
pub mod message;
pub mod report;
pub mod role;
pub mod user;

#[derive(MergedObject, Default)]
pub struct Query(
	cosmetic::CosmeticsQuery,
	emote_set::EmoteSetsQuery,
	emote::EmotesQuery,
	message::MessagesQuery,
	report::ReportsQuery,
	role::RolesQuery,
	user::UsersQuery,
);
