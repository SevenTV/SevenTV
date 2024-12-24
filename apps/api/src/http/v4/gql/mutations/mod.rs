use shared::database::user::UserId;

mod billing;
mod emote;
mod emote_set;
mod ticket;
mod user;
mod user_editor;
mod user_session;

#[derive(async_graphql::SimpleObject, Default)]
#[graphql(complex)]
pub struct Mutation {
	emotes: emote::EmoteMutation,
	emote_sets: emote_set::EmoteSetMutation,
	tickets: ticket::TicketMutation,
	users: user::UserMutation,
	user_editors: user_editor::UserEditorMutation,
	user_sessions: user_session::UserSessionMutation,
}

#[async_graphql::ComplexObject]
impl Mutation {
	async fn billing(&self, user_id: UserId) -> billing::BillingMutation {
		billing::BillingMutation { user_id }
	}
}
