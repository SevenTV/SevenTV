use async_graphql::SimpleObject;

mod backdoor;
mod emote;
mod emote_set;
mod search;
mod store;
mod user;

#[derive(Default, SimpleObject)]
pub struct Query {
	emotes: emote::EmoteQuery,
	emote_sets: emote_set::EmoteSetQuery,
	search: search::SearchQuery,
	store: store::StoreQuery,
	backdoor: backdoor::BackdoorQuery,
	users: user::UserQuery,
}
