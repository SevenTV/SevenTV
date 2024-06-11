// https://github.com/SevenTV/API/tree/main/internal/api/gql/v3/schema

use async_graphql::MergedObject;

mod bans;
mod cosmetics;
mod emote_sets;
mod emotes;
mod messages;
mod reports;
mod roles;
mod users;

#[derive(MergedObject, Default)]
pub struct Mutation(
	bans::BansMutation,
	cosmetics::CosmeticsMutation,
	emote_sets::EmoteSetsMutation,
	emotes::EmotesMutation,
	messages::MessagesMutation,
	reports::ReportsMutation,
	roles::RolesMutation,
	users::UsersMutation,
);
