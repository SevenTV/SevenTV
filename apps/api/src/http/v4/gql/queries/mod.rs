mod backdoor;
mod emote;
mod emote_set;
mod product;
mod search;
mod store;
mod user;

#[derive(Default, async_graphql::SimpleObject)]
pub struct Query {
	backdoor: backdoor::BackdoorQuery,
	emotes: emote::EmoteQuery,
	emote_sets: emote_set::EmoteSetQuery,
	products: product::ProductQuery,
	search: search::SearchQuery,
	store: store::StoreQuery,
	users: user::UserQuery,
}
