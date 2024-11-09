use async_graphql::SimpleObject;

mod emote;
mod emote_set;
mod search;
mod user;

#[derive(Default, SimpleObject)]
pub struct Query {
	emotes: emote::EmoteQuery,
	emote_sets: emote_set::EmoteSetQuery,
	search: search::SearchQuery,
	users: user::UserQuery,
}
