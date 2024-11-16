mod emote;

#[derive(async_graphql::MergedObject, Default)]
pub struct Mutation(
	emote::EmoteMutation,
);
