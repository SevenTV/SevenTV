use async_graphql::SimpleObject;

mod emote;
mod user;

#[derive(Default, SimpleObject)]
pub struct Query {
    emotes: emote::EmoteQuery,
	users: user::UserQuery,
}
