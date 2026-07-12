use async_graphql::Context;
use shared::database::product::StripeProductId;
use shared::database::role::permissions::RateLimitResource;
use shared::database::user::UserId;

use crate::http::error::ApiError;
use crate::http::guards::RateLimitGuard;

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

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EgVaultSubscribe, 1)")]
	#[tracing::instrument(skip_all, name = "Mutation::gift_subscriptions")]
	async fn gift_subscriptions(
		&self,
		ctx: &Context<'_>,
		recipients: Vec<billing::GiftRecipientInput>,
		variant_id: StripeProductId,
	) -> Result<billing::SubscribeResponse, ApiError> {
		billing::gift_subscriptions(ctx, recipients, variant_id).await
	}
}
