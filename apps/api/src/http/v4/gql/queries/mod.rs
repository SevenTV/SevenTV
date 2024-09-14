use async_graphql::SimpleObject;

mod emote;

#[derive(Default, SimpleObject)]
pub struct Query {
    emotes: emote::EmoteQuery,
}
