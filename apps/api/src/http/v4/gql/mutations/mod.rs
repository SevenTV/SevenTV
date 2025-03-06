use shared::database::user::UserId;

mod billing;
mod emote;
mod emote_set;
mod entitlement_edge;
mod jobs;
mod product;
mod redeem_code;
mod special_event;
mod ticket;
mod user;
mod user_editor;
mod user_session;

#[derive(async_graphql::SimpleObject, Default)]
#[graphql(complex)]
pub struct Mutation {
	emotes: emote::EmoteMutation,
	emote_sets: emote_set::EmoteSetMutation,
	entitlement_edges: entitlement_edge::EntitlementEdgeMutation,
	jobs: jobs::JobMutation,
	redeem_codes: redeem_code::RedeemCodeMutation,
	special_events: special_event::SpecialEventMutation,
	product: product::ProductMutation,
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
