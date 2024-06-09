use async_graphql::MergedObject;

// https://github.com/SevenTV/API/tree/main/internal/api/gql/v3/schema

mod audit_logs;
mod cosmetics;
mod emote_sets;
mod emotes;
mod messages;
mod reports;
mod roles;
mod users;

pub use audit_logs::*;
pub use cosmetics::*;
pub use emote_sets::*;
pub use emotes::*;
pub use messages::*;
pub use reports::*;
pub use roles::*;
pub use users::*;

#[derive(MergedObject, Default)]
pub struct Query(
	cosmetics::CosmeticsQuery,
	emote_sets::EmoteSetsQuery,
	emotes::EmotesQuery,
	messages::MessagesQuery,
	reports::ReportsQuery,
	roles::RolesQuery,
	users::UsersQuery,
);
