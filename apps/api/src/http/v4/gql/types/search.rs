use async_graphql::{OutputType, SimpleObject};

use super::{Emote, EmoteSetEmote, RedeemCode, User};

#[derive(SimpleObject)]
#[graphql(concrete(name = "UserSearchResult", params(User)))]
#[graphql(concrete(name = "EmoteSearchResult", params(Emote)))]
#[graphql(concrete(name = "EmoteSetEmoteSearchResult", params(EmoteSetEmote)))]
#[graphql(concrete(name = "RedeemCodeSearchResult", params(RedeemCode)))]
pub struct SearchResult<T: OutputType> {
	pub items: Vec<T>,
	pub total_count: u64,
	pub page_count: u64,
}
