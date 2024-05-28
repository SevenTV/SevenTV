use async_graphql::MergedObject;

mod audit_logs;
mod emote_sets;
mod emotes;
mod reports;
mod users;

#[derive(MergedObject, Default)]
pub struct Query(
	emotes::EmotesQuery,
	emote_sets::EmoteSetsQuery,
	reports::ReportsQuery,
	users::UsersQuery,
);
