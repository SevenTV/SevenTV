mod backdoor;
mod badge;
mod emote;
mod emote_set;
mod entitlement;
mod paint;
mod product;
mod redeem_code;
mod role;
mod search;
mod special_event;
mod store;
mod user;

#[derive(Default, async_graphql::SimpleObject)]
pub struct Query {
	backdoor: backdoor::BackdoorQuery,
	badges: badge::BadgeQuery,
	emotes: emote::EmoteQuery,
	emote_sets: emote_set::EmoteSetQuery,
	entitlements: entitlement::EntitlementQuery,
	paints: paint::PaintQuery,
	products: product::ProductQuery,
	redeem_codes: redeem_code::RedeemCodeQuery,
	roles: role::RoleQuery,
	search: search::SearchQuery,
	special_events: special_event::SpecialEventQuery,
	store: store::StoreQuery,
	users: user::UserQuery,
}
