mod backdoor;
mod emote;
mod emote_set;
mod product;
mod redeem_code;
mod search;
mod special_event;
mod store;
mod user;

#[derive(Default, async_graphql::SimpleObject)]
pub struct Query {
	backdoor: backdoor::BackdoorQuery,
	emotes: emote::EmoteQuery,
	emote_sets: emote_set::EmoteSetQuery,
	products: product::ProductQuery,
	redeem_codes: redeem_code::RedeemCodeQuery,
	search: search::SearchQuery,
	special_events: special_event::SpecialEventQuery,
	store: store::StoreQuery,
	users: user::UserQuery,
}
