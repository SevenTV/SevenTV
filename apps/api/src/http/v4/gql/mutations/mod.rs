mod emote;
mod emote_set;

#[derive(async_graphql::MergedObject, Default)]
pub struct Mutation(emote::EmoteMutation, emote_set::EmoteSetMutation);
